//! Whole-scene comparison against normalized Steam files: exact, shape, authored or structural.
//!
//! Exact requires identical instructions and operands. Shape permits source-numeric
//! drift only at tracked opcode-`0x00` pushes; fixed values, strings, targets and
//! opcodes stay exact. Authored applies reviewed manifest edits before the shape
//! check. Structural means the scene still differs after those edits.
//!
//! ```
//! use hunex_chs::compare::{compare, SceneClass, SceneInput};
//! use hunex_chs::normalize::NormInstruction;
//!
//! let list = [NormInstruction::new(0x1B, Vec::new())];
//! let input = SceneInput {
//!     expected: &list,
//!     actual: &list,
//!     slots: &[],
//!     owners: &[],
//!     switch_chars: &[],
//!     steam_offsets: &[],
//!     pool_start: 0,
//!     has_problems: false,
//!     authored: &[],
//! };
//! assert_eq!(compare(&input).class, SceneClass::Exact);
//! ```

use alloc::vec::Vec;

use crate::normalize::{NormInstruction, NormOperand};

/// Whole-scene classification, ordered from worst to best.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SceneClass {
    /// The scene still differs after reviewed authored edits.
    Structural,
    /// The scene matches only with reviewed authored edits applied.
    Authored,
    /// The scene matches with source-numeric value drift.
    Shape,
    /// The scene matches instruction for instruction.
    Exact,
}

impl SceneClass {
    /// The per-scene label for the class, as printed in corpus reports.
    pub fn label(self) -> &'static str {
        match self {
            SceneClass::Exact => "exact match",
            SceneClass::Shape => "shape match with value drift",
            SceneClass::Authored => "authored",
            SceneClass::Structural => "structural mismatch",
        }
    }
}

/// One reviewed authored difference, parsed by the caller from the manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredDiff<'a> {
    /// The source instruction index the replaced fragment starts at.
    pub source_index: usize,
    /// The Steam instruction index the replacement starts at.
    pub steam_index: usize,
    /// The exact expected source fragment.
    pub source_value: &'a [NormInstruction],
    /// The exact reviewed Steam replacement fragment.
    pub steam_value: &'a [NormInstruction],
    /// The Switch character offset owning the source fragment, or `None` for the
    /// prologue and epilogue.
    pub switch_char: Option<usize>,
    /// The Steam byte offset of the Steam fragment.
    pub steam_offset: usize,
    /// The entry was reviewed.
    pub reviewed: bool,
    /// The entry carries nonempty evidence.
    pub has_evidence: bool,
}

/// Everything one scene comparison needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneInput<'a> {
    /// The compiled normalized instructions.
    pub expected: &'a [NormInstruction],
    /// The Steam normalized instructions.
    pub actual: &'a [NormInstruction],
    /// The compiled instruction indices that may drift as source numerics.
    pub slots: &'a [usize],
    /// The owning statement of each compiled instruction (`-1` prologue, `-2` tail).
    pub owners: &'a [i64],
    /// The Switch character offset of each source statement.
    pub switch_chars: &'a [usize],
    /// The Steam byte offset of each normalized Steam instruction.
    pub steam_offsets: &'a [usize],
    /// The Steam pool start, used when an offset runs past the last instruction.
    pub pool_start: usize,
    /// The compilation reported diagnostics, which prevent any acceptance.
    pub has_problems: bool,
    /// The reviewed authored entries for the scene.
    pub authored: &'a [AuthoredDiff<'a>],
}

/// Why authored comparison failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthoredFailure {
    /// An entry is unreviewed, unevidenced, stale or out of order.
    StaleOrUnreviewed,
    /// A prior unlisted structural change breaks the patched correspondence.
    UnlistedEdits,
    /// A surviving target points into a replaced interior.
    TargetEntersInterior,
}

/// The result of one scene comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneComparison {
    /// The strongest comparison level the scene satisfies.
    pub class: SceneClass,
    /// The first exact divergence index, or the shorter length on truncation.
    pub exact_divergence: Option<usize>,
    /// The first shape divergence index, or the shorter length on truncation.
    pub shape_divergence: Option<usize>,
    /// Whether the scene matches with reviewed authored edits applied.
    pub authored_matching: bool,
    /// Why authored comparison failed, when it did.
    pub authored_failure: Option<AuthoredFailure>,
}

/// True when the two instructions are equal, relaxing a tracked source numeric
/// push under shape comparison.
fn instructions_equal(expected: &NormInstruction, actual: &NormInstruction, relaxed: bool) -> bool {
    if expected == actual {
        return true;
    }
    if !relaxed || expected.opcode() != 0 || actual.opcode() != 0 {
        return false;
    }
    matches!(
        (expected.operands(), actual.operands()),
        ([NormOperand::Num(_)], [NormOperand::Num(_)])
    )
}

/// The first index where the two lists differ under the given relaxation, or the
/// shorter length when one list truncates the other. `slots` must be sorted.
fn first_divergence(
    expected: &[NormInstruction],
    actual: &[NormInstruction],
    slots: &[usize],
) -> Option<usize> {
    let common = expected.len().min(actual.len());
    for index in 0..common {
        let relaxed = slots.binary_search(&index).is_ok();
        if !instructions_equal(&expected[index], &actual[index], relaxed) {
            return Some(index);
        }
    }
    if expected.len() != actual.len() {
        return Some(common);
    }
    None
}

/// The patched stream plus, for each patched instruction, the source index it came
/// from (`None` for replacement instructions).
struct Patched {
    code: Vec<NormInstruction>,
    origin: Vec<Option<usize>>,
}

/// Replays the reviewed authored entries over the compiled stream: untouched
/// instructions keep their source index, replacement instructions are taken
/// exactly as recorded, and untouched `PushOffset` targets relocate through the
/// surviving source-to-patched map.
fn apply_authored(input: &SceneInput<'_>) -> Result<Patched, AuthoredFailure> {
    let mut order: Vec<usize> = (0..input.authored.len()).collect();
    order.sort_by_key(|position| {
        let entry = &input.authored[*position];
        (entry.source_index, entry.steam_index)
    });
    let mut patched: Vec<NormInstruction> = Vec::new();
    let mut from_source: Vec<Option<usize>> = Vec::new();
    let mut mapping: Vec<(usize, usize)> = Vec::new();
    let mut origin: Vec<(usize, usize)> = Vec::new();
    let mut cursor = 0usize;
    // The map keys are pushed in ascending order, so relocation can binary-search.
    let map = |table: &[(usize, usize)], key: usize| -> Option<usize> {
        table
            .binary_search_by(|(from, _)| from.cmp(&key))
            .ok()
            .map(|position| table[position].1)
    };
    for position in order {
        let entry = &input.authored[position];
        let owner = if entry.source_index < input.expected.len() {
            input.owners.get(entry.source_index).copied().unwrap_or(-2)
        } else {
            -2
        };
        let switch_char = if owner >= 0 {
            input
                .switch_chars
                .get(usize::try_from(owner).unwrap_or(usize::MAX))
                .copied()
        } else {
            None
        };
        let steam_offset = input
            .steam_offsets
            .get(entry.steam_index)
            .copied()
            .unwrap_or(input.pool_start);
        let end = entry
            .source_index
            .checked_add(entry.source_value.len())
            .ok_or(AuthoredFailure::StaleOrUnreviewed)?;
        let source_matches = end <= input.expected.len()
            && input.expected[entry.source_index..end] == *entry.source_value;
        let steam_end = entry
            .steam_index
            .checked_add(entry.steam_value.len())
            .ok_or(AuthoredFailure::StaleOrUnreviewed)?;
        let steam_matches = steam_end <= input.actual.len()
            && input.actual[entry.steam_index..steam_end] == *entry.steam_value;
        if !entry.reviewed
            || !entry.has_evidence
            || entry.source_index < cursor
            || !source_matches
            || !steam_matches
            || switch_char != entry.switch_char
            || steam_offset != entry.steam_offset
        {
            return Err(AuthoredFailure::StaleOrUnreviewed);
        }
        for index in cursor..entry.source_index {
            mapping.push((index, patched.len()));
            origin.push((patched.len(), index));
            from_source.push(Some(index));
            patched.push(input.expected[index].clone());
        }
        if patched.len() != entry.steam_index {
            return Err(AuthoredFailure::UnlistedEdits);
        }
        mapping.push((entry.source_index, patched.len()));
        for replacement in entry.steam_value {
            from_source.push(None);
            patched.push(replacement.clone());
        }
        cursor = end;
    }
    for index in cursor..input.expected.len() {
        mapping.push((index, patched.len()));
        origin.push((patched.len(), index));
        from_source.push(Some(index));
        patched.push(input.expected[index].clone());
    }
    mapping.push((input.expected.len(), patched.len()));
    for (patched_index, _) in &origin {
        if patched[*patched_index].opcode() == 0x01 {
            let mut relocated = patched[*patched_index].operands().to_vec();
            for operand in &mut relocated {
                if let NormOperand::Target(target) = operand {
                    let mapped =
                        map(&mapping, *target).ok_or(AuthoredFailure::TargetEntersInterior)?;
                    *target = mapped;
                }
            }
            patched[*patched_index] =
                NormInstruction::new(patched[*patched_index].opcode(), relocated);
        }
    }
    Ok(Patched {
        code: patched,
        origin: from_source,
    })
}

/// Classifies one scene: exact, then shape, then reviewed-authored comparison.
pub fn compare(input: &SceneInput<'_>) -> SceneComparison {
    let mut slots: Vec<usize> = input.slots.to_vec();
    slots.sort_unstable();
    slots.dedup();
    let exact_divergence = first_divergence(input.expected, input.actual, &[]);
    let exact = exact_divergence.is_none() && !input.has_problems;
    let shape_divergence = first_divergence(input.expected, input.actual, &slots);
    let shape = shape_divergence.is_none() && !input.has_problems;
    let (authored_matching, authored_failure) = match apply_authored(input) {
        Err(failure) => (false, Some(failure)),
        Ok(patched) => {
            let mut first = None;
            for (index, patched_instr) in patched.code.iter().enumerate() {
                match input.actual.get(index) {
                    None => {
                        first = Some(index);
                        break;
                    }
                    Some(actual_instr) if patched_instr == actual_instr => {}
                    Some(actual_instr) => {
                        let relaxed = patched.origin[index]
                            .is_some_and(|source| slots.binary_search(&source).is_ok());
                        if !instructions_equal(patched_instr, actual_instr, relaxed) {
                            first = Some(index);
                            break;
                        }
                    }
                }
            }
            if first.is_none() && patched.code.len() != input.actual.len() {
                first = Some(patched.code.len().min(input.actual.len()));
            }
            let matching = first.is_none() && !input.has_problems;
            (matching, None)
        }
    };
    let class = if exact {
        SceneClass::Exact
    } else if shape {
        SceneClass::Shape
    } else if authored_matching {
        SceneClass::Authored
    } else {
        SceneClass::Structural
    };
    SceneComparison {
        class,
        exact_divergence,
        shape_divergence,
        authored_matching,
        authored_failure,
    }
}
