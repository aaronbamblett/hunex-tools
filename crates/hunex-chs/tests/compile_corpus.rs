//! Gated corpus checks for the Switch compiler: every Switch script parses with the
//! expected statement totals, every Steam `.chs` file normalizes cleanly, and all
//! 249 scenes compile to the exact, shape and authored classes listed in the
//! expected-classes file supplied via `HUNEX_COMPILE_EXPECTED_CLASSES`.
//!
//! | Variable | Points at | Unset behavior |
//! |---|---|---|
//! | `HUNEX_SWITCH_SCRIPT_DIR` | The Switch dump folder (`scr0000.txt`..`scr0248.txt`). | The script test is skipped. |
//! | `HUNEX_CHS_DIR` | A folder of Steam `.chs` files. | The normalize test is skipped. |
//! | `HUNEX_COMPILE_EXPECTED` | The `expected-counts.json` statement totals. | The per-scene comparison is skipped. |
//! | `HUNEX_COMPILE_SCENES` | The `scene-order.json` fixture: 249 scene names in `scr` order. | The compile test is skipped. |
//! | `HUNEX_COMPILE_SCENE_NUMBERS` | The `scene-numbers.json` fixture: `_JUMP` number to Steam name. | The compile test is skipped. |
//! | `HUNEX_COMPILE_AUTHORED` | The reviewed `authored-diffs-v2.json` manifest. | The compile test is skipped. |
//! | `HUNEX_COMPILE_EXPECTED_CLASSES` | The `expected-classes.json` fixture: expected class per scene. | The compile test is skipped. |

use std::collections::BTreeMap;
use std::path::PathBuf;

use hunex_chs::{compare, compile, normalize, source, Script};

const SCENES: usize = 249;
const EXPECTED_STATEMENTS: usize = 493_227;
const EXPECTED_MALFORMED: usize = 14;
const EXPECTED_CHS_FILES: usize = 250;

const DEBUG: u32 = 0x7F;
const PUSH_OFFSET: u32 = 0x01;

fn switch_scripts() -> Option<Vec<PathBuf>> {
    let Some(dir) = std::env::var_os("HUNEX_SWITCH_SCRIPT_DIR") else {
        eprintln!("HUNEX_SWITCH_SCRIPT_DIR is not set; skipping Switch script checks");
        return None;
    };
    let dir = PathBuf::from(dir);
    let mut files = Vec::with_capacity(SCENES);
    for number in 0..SCENES {
        files.push(dir.join(format!("scr{number:04}.txt")));
    }
    Some(files)
}

fn chs_files() -> Option<Vec<PathBuf>> {
    let Some(dir) = std::env::var_os("HUNEX_CHS_DIR") else {
        eprintln!("HUNEX_CHS_DIR is not set; skipping normalize checks");
        return None;
    };
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("read HUNEX_CHS_DIR")
        .map(|entry| entry.expect("directory entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "chs"))
        .collect();
    files.sort();
    Some(files)
}

#[test]
fn switch_scripts_parse_with_expected_totals() {
    let Some(files) = switch_scripts() else {
        return;
    };
    assert_eq!(files.len(), SCENES);

    let mut total = 0usize;
    let mut malformed = 0usize;
    let mut scenes: Vec<(Vec<String>, Vec<usize>)> = Vec::with_capacity(SCENES);
    for path in &files {
        let name = path.display();
        let text = std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{name}: {error}"));
        let statements = source::parse(&text).unwrap_or_else(|error| panic!("{name}: {error}"));
        total += statements.len();
        malformed += statements.iter().filter(|item| item.malformed()).count();
        scenes.push((
            statements
                .iter()
                .map(|item| item.name().to_owned())
                .collect(),
            statements.iter().map(|item| item.offset()).collect(),
        ));
    }
    println!(
        "parsed {} Switch scripts: {total} statements, {malformed} malformed _KWRS",
        files.len()
    );
    assert_eq!(total, EXPECTED_STATEMENTS, "total statement count");
    assert_eq!(malformed, EXPECTED_MALFORMED, "malformed _KWRS count");

    let Some(path) = std::env::var_os("HUNEX_COMPILE_EXPECTED") else {
        eprintln!("HUNEX_COMPILE_EXPECTED is not set; skipping per-scene comparison");
        return;
    };
    let expected = std::fs::read_to_string(&path).expect("read HUNEX_COMPILE_EXPECTED");
    let expected = Expected::parse(&expected).expect("parse HUNEX_COMPILE_EXPECTED");
    assert_eq!(
        expected.scenes.len(),
        SCENES,
        "expected-counts.json scene count"
    );
    for (index, (names, offsets)) in scenes.iter().enumerate() {
        let scene = &expected.scenes[index];
        assert_eq!(names.len(), scene.count, "scene {} count", scene.name);
        assert_eq!(&scene.cmds, names, "scene {} names", scene.name);
        assert_eq!(&scene.positions, offsets, "scene {} offsets", scene.name);
    }
    println!("compared per-statement names and offsets for {SCENES} scenes");
}

/// One scene row of `expected-counts.json`.
struct ExpectedScene {
    name: String,
    count: usize,
    cmds: Vec<String>,
    positions: Vec<usize>,
}

/// The parsed `expected-counts.json`: scene rows in file order.
struct Expected {
    scenes: Vec<ExpectedScene>,
}

impl Expected {
    /// Parses the exact shape the exporter writes: an object with a `scenes` array of
    /// objects holding `scene`, `count`, `malformed`, `cmds` and `positions`.
    fn parse(text: &str) -> Result<Expected, String> {
        let mut reader = JsonReader {
            chars: &text.chars().collect::<Vec<_>>(),
            pos: 0,
        };
        let value = reader.value()?;
        reader.end()?;
        let Value::Object(root) = value else {
            return Err("root is not an object".to_owned());
        };
        let Some(Value::Array(scenes)) = root
            .iter()
            .find_map(|(key, value)| (*key == "scenes").then(|| value.clone()))
        else {
            return Err("missing scenes array".to_owned());
        };
        let mut out = Vec::with_capacity(scenes.len());
        for scene in scenes {
            let Value::Object(fields) = scene else {
                return Err("scene is not an object".to_owned());
            };
            let get = |key: &str| {
                fields
                    .iter()
                    .find(|(name, _)| *name == key)
                    .map(|(_, value)| value.clone())
                    .ok_or_else(|| format!("scene is missing {key}"))
            };
            let Value::String(name) = get("scene")? else {
                return Err("scene name is not a string".to_owned());
            };
            let Value::Number(count) = get("count")? else {
                return Err("scene count is not a number".to_owned());
            };
            let Value::Array(cmds) = get("cmds")? else {
                return Err("scene cmds is not an array".to_owned());
            };
            let Value::Array(positions) = get("positions")? else {
                return Err("scene positions is not an array".to_owned());
            };
            let mut names = Vec::with_capacity(cmds.len());
            for cmd in cmds {
                let Value::String(cmd) = cmd else {
                    return Err("scene cmd is not a string".to_owned());
                };
                names.push(cmd);
            }
            let mut offsets = Vec::with_capacity(positions.len());
            for position in positions {
                let Value::Number(position) = position else {
                    return Err("scene position is not a number".to_owned());
                };
                offsets.push(position as usize);
            }
            out.push(ExpectedScene {
                name,
                count: count as usize,
                cmds: names,
                positions: offsets,
            });
        }
        Ok(Expected { scenes: out })
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Value {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
}

impl Value {
    fn get(&self, key: &str) -> Result<Value, String> {
        let Value::Object(fields) = self else {
            return Err("value is not an object".to_owned());
        };
        fields
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.clone())
            .ok_or_else(|| format!("object is missing {key}"))
    }

    fn expect_string(&self) -> Result<String, String> {
        match self {
            Value::String(text) => Ok(text.clone()),
            _ => Err("value is not a string".to_owned()),
        }
    }

    fn expect_usize(&self) -> Result<usize, String> {
        match self {
            Value::Number(number) => Ok(*number as usize),
            _ => Err("value is not a number".to_owned()),
        }
    }
}

struct JsonReader<'a> {
    chars: &'a [char],
    pos: usize,
}

impl JsonReader<'_> {
    fn end(&mut self) -> Result<(), String> {
        self.skip_space();
        if self.pos == self.chars.len() {
            Ok(())
        } else {
            Err(format!("trailing text at char {}", self.pos))
        }
    }

    fn skip_space(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn value(&mut self) -> Result<Value, String> {
        self.skip_space();
        match self.chars.get(self.pos) {
            Some(&'"') => Ok(Value::String(self.string()?)),
            Some(&'{') => Ok(Value::Object(self.object()?)),
            Some(&'[') => Ok(Value::Array(self.array()?)),
            Some(_) => {
                for literal in ["null", "true", "false"] {
                    if self.ahead(literal) {
                        self.pos += literal.len();
                        return Ok(match literal {
                            "null" => Value::Null,
                            "true" => Value::Bool(true),
                            _ => Value::Bool(false),
                        });
                    }
                }
                self.number()
            }
            None => Err("unexpected end of input".to_owned()),
        }
    }

    fn ahead(&self, literal: &str) -> bool {
        let wanted: Vec<char> = literal.chars().collect();
        self.chars.len() - self.pos >= wanted.len()
            && self.chars[self.pos..self.pos + wanted.len()] == wanted
    }

    fn string(&mut self) -> Result<String, String> {
        self.pos += 1; // Skip the opening quote.
        let mut out = String::new();
        loop {
            match self.chars.get(self.pos) {
                None => return Err("unterminated string".to_owned()),
                Some(&'"') => {
                    self.pos += 1;
                    return Ok(out);
                }
                Some(&'\\') => {
                    self.pos += 1;
                    match self.chars.get(self.pos) {
                        Some(&'"') => out.push('"'),
                        Some(&'\\') => out.push('\\'),
                        Some(&'/') => out.push('/'),
                        Some(&'b') => out.push('\u{8}'),
                        Some(&'f') => out.push('\u{c}'),
                        Some(&'n') => out.push('\n'),
                        Some(&'r') => out.push('\r'),
                        Some(&'t') => out.push('\t'),
                        Some(&'u') => {
                            if self.pos + 4 >= self.chars.len() {
                                return Err("unterminated escape".to_owned());
                            }
                            let digits: String =
                                self.chars[self.pos + 1..self.pos + 5].iter().collect();
                            let code = u32::from_str_radix(&digits, 16)
                                .map_err(|_| "bad unicode escape".to_owned())?;
                            out.push(char::from_u32(code).ok_or("bad unicode escape".to_owned())?);
                            self.pos += 4;
                        }
                        _ => return Err("bad escape".to_owned()),
                    }
                    self.pos += 1;
                }
                Some(&cell) => {
                    out.push(cell);
                    self.pos += 1;
                }
            }
        }
    }

    fn number(&mut self) -> Result<Value, String> {
        let start = self.pos;
        if self.chars.get(self.pos) == Some(&'-') {
            self.pos += 1;
        }
        while self.pos < self.chars.len()
            && (self.chars[self.pos].is_ascii_digit() || "+-.eE".contains(self.chars[self.pos]))
        {
            self.pos += 1;
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        let number: f64 = text.parse().map_err(|_| format!("bad number {text:?}"))?;
        Ok(Value::Number(number))
    }

    fn array(&mut self) -> Result<Vec<Value>, String> {
        self.pos += 1; // Skip `[`.
        let mut out = Vec::new();
        self.skip_space();
        if self.chars.get(self.pos) == Some(&']') {
            self.pos += 1;
            return Ok(out);
        }
        loop {
            out.push(self.value()?);
            self.skip_space();
            match self.chars.get(self.pos) {
                Some(&',') => {
                    self.pos += 1;
                }
                Some(&']') => {
                    self.pos += 1;
                    return Ok(out);
                }
                _ => return Err(format!("bad array at char {}", self.pos)),
            }
        }
    }

    fn object(&mut self) -> Result<Vec<(String, Value)>, String> {
        self.pos += 1; // Skip `{`.
        let mut out = Vec::new();
        self.skip_space();
        if self.chars.get(self.pos) == Some(&'}') {
            self.pos += 1;
            return Ok(out);
        }
        loop {
            self.skip_space();
            if self.chars.get(self.pos) != Some(&'"') {
                return Err(format!("bad object key at char {}", self.pos));
            }
            let key = self.string()?;
            self.skip_space();
            if self.chars.get(self.pos) != Some(&':') {
                return Err(format!("missing colon at char {}", self.pos));
            }
            self.pos += 1;
            out.push((key, self.value()?));
            self.skip_space();
            match self.chars.get(self.pos) {
                Some(&',') => {
                    self.pos += 1;
                }
                Some(&'}') => {
                    self.pos += 1;
                    return Ok(out);
                }
                _ => return Err(format!("bad object at char {}", self.pos)),
            }
        }
    }
}

#[test]
fn steam_files_normalize_without_debug() {
    let Some(files) = chs_files() else {
        return;
    };
    assert_eq!(files.len(), EXPECTED_CHS_FILES);

    for path in &files {
        let name = path.display();
        let bytes = std::fs::read(path).unwrap_or_else(|error| panic!("{name}: {error}"));
        let script = Script::parse(&bytes).unwrap_or_else(|error| panic!("{name}: {error}"));
        let program = script
            .lift()
            .unwrap_or_else(|error| panic!("{name}: lift: {error}"));
        let list = normalize::normalize(&program)
            .unwrap_or_else(|error| panic!("{name}: normalize: {error}"));
        for item in &list {
            assert_ne!(
                item.opcode(),
                DEBUG,
                "{name}: Debug remains after normalize"
            );
            if item.opcode() == PUSH_OFFSET {
                match item.operands() {
                    [normalize::NormOperand::Target(target)] => assert!(
                        *target <= list.len(),
                        "{name}: PushOffset target {target} past the end of the list",
                    ),
                    operands => {
                        panic!("{name}: PushOffset operands are not one target: {operands:?}")
                    }
                }
            }
        }
    }
    println!("normalized {} Steam .chs files", files.len());
}

const EXPECTED_EXACT: usize = 162;
const EXPECTED_SHAPE: usize = 216;
const EXPECTED_AUTHORED: usize = 242;

fn read_fixture(name: &str) -> Option<String> {
    let path = std::env::var_os(name).unwrap_or_else(|| {
        eprintln!("{name} is not set; skipping compile comparison checks");
        std::ffi::OsString::new()
    });
    if path.is_empty() {
        return None;
    }
    Some(std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {name}: {error}")))
}

fn parse_json(text: &str) -> Result<Value, String> {
    let mut reader = JsonReader {
        chars: &text.chars().collect::<Vec<_>>(),
        pos: 0,
    };
    let value = reader.value()?;
    reader.end()?;
    Ok(value)
}

/// Parses one normalized instruction array from a fixture: numbers stay numbers,
/// strings stay strings, and a lone operand of opcode `0x01` is a jump target.
fn parse_norm_instruction(
    value: &Value,
    scene: &str,
) -> Result<normalize::NormInstruction, String> {
    let Value::Array(items) = value else {
        return Err(format!("{scene}: instruction is not an array"));
    };
    let [Value::Number(opcode), rest @ ..] = items.as_slice() else {
        return Err(format!("{scene}: instruction has no opcode"));
    };
    let opcode = *opcode as u32;
    let mut operands = Vec::with_capacity(rest.len());
    for operand in rest {
        match operand {
            Value::Number(number) if opcode == 0x01 && rest.len() == 1 => {
                operands.push(normalize::NormOperand::Target(*number as usize));
            }
            Value::Number(number) => operands.push(normalize::NormOperand::Num(*number as u32)),
            Value::String(text) => {
                operands.push(normalize::NormOperand::Text(text.as_bytes().to_vec()))
            }
            _ => return Err(format!("{scene}: bad instruction operand")),
        }
    }
    Ok(normalize::NormInstruction::new(opcode, operands))
}

fn class_rank(class: &compare::SceneClass) -> u8 {
    match class {
        compare::SceneClass::Structural => 0,
        compare::SceneClass::Authored => 1,
        compare::SceneClass::Shape => 2,
        compare::SceneClass::Exact => 3,
    }
}

fn class_of_label(label: &str) -> Result<compare::SceneClass, String> {
    match label {
        "exact match" => Ok(compare::SceneClass::Exact),
        "shape match with value drift" => Ok(compare::SceneClass::Shape),
        "authored" => Ok(compare::SceneClass::Authored),
        "structural mismatch" => Ok(compare::SceneClass::Structural),
        _ => Err(format!("unknown class label {label:?}")),
    }
}

#[test]
fn switch_scenes_compile_to_reference_classes() {
    let Some(switch_dir) = std::env::var_os("HUNEX_SWITCH_SCRIPT_DIR") else {
        eprintln!("HUNEX_SWITCH_SCRIPT_DIR is not set; skipping compile comparison checks");
        return;
    };
    let Some(chs_dir) = std::env::var_os("HUNEX_CHS_DIR") else {
        eprintln!("HUNEX_CHS_DIR is not set; skipping compile comparison checks");
        return;
    };
    let (Some(scenes_text), Some(numbers_text), Some(authored_text), Some(classes_text)) = (
        read_fixture("HUNEX_COMPILE_SCENES"),
        read_fixture("HUNEX_COMPILE_SCENE_NUMBERS"),
        read_fixture("HUNEX_COMPILE_AUTHORED"),
        read_fixture("HUNEX_COMPILE_EXPECTED_CLASSES"),
    ) else {
        eprintln!(
            "HUNEX_COMPILE_SCENES, HUNEX_COMPILE_SCENE_NUMBERS, HUNEX_COMPILE_AUTHORED and \
             HUNEX_COMPILE_EXPECTED_CLASSES are not all set; skipping compile comparison checks"
        );
        return;
    };

    let Value::Array(scenes) = parse_json(&scenes_text).expect("parse HUNEX_COMPILE_SCENES") else {
        panic!("HUNEX_COMPILE_SCENES root is not an array");
    };
    let scene_names: Vec<String> = scenes
        .iter()
        .map(|scene| scene.expect_string().expect("scene name is not a string"))
        .collect();
    assert_eq!(scene_names.len(), SCENES, "scene count");

    let Value::Object(numbers) =
        parse_json(&numbers_text).expect("parse HUNEX_COMPILE_SCENE_NUMBERS")
    else {
        panic!("HUNEX_COMPILE_SCENE_NUMBERS root is not an object");
    };
    let scene_numbers: Vec<(String, String)> = numbers
        .iter()
        .map(|(number, name)| {
            (
                number.clone(),
                name.expect_string()
                    .expect("scene number target is not a string"),
            )
        })
        .collect();
    let number_refs: Vec<(&str, &str)> = scene_numbers
        .iter()
        .map(|(number, name)| (number.as_str(), name.as_str()))
        .collect();

    let authored_root = parse_json(&authored_text).expect("parse HUNEX_COMPILE_AUTHORED");
    let Value::Array(differences) = authored_root
        .get("differences")
        .expect("authored differences")
    else {
        panic!("authored differences is not an array");
    };

    let Value::Object(expected_classes) =
        parse_json(&classes_text).expect("parse HUNEX_COMPILE_EXPECTED_CLASSES")
    else {
        panic!("HUNEX_COMPILE_EXPECTED_CLASSES root is not an object");
    };
    assert_eq!(expected_classes.len(), SCENES, "expected class count");

    let switch_dir = PathBuf::from(switch_dir);
    let chs_dir = PathBuf::from(chs_dir);
    let mut exact = 0usize;
    let mut shape = 0usize;
    let mut authored_count = 0usize;
    let mut beats: Vec<String> = Vec::new();
    let mut worse: Vec<String> = Vec::new();
    let mut compile_failures: Vec<String> = Vec::new();
    let mut warning_total = 0usize;
    let mut warnings_by_command: BTreeMap<String, usize> = BTreeMap::new();

    for (number, scene) in scene_names.iter().enumerate() {
        let path = switch_dir.join(format!("scr{number:04}.txt"));
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let statements =
            source::parse(&text).unwrap_or_else(|error| panic!("{scene}: parse: {error}"));
        let switch_chars: Vec<usize> = statements.iter().map(|item| item.offset()).collect();
        let compiled = match compile::compile(&statements, &number_refs) {
            Ok(compiled) => compiled,
            Err(error) => {
                compile_failures.push(format!("{scene}: {error}"));
                let expected = expected_classes
                    .iter()
                    .find(|(name, _)| name == scene)
                    .map(|(_, class)| class.expect_string().unwrap_or_default())
                    .unwrap_or_default();
                worse.push(format!(
                    "{scene}: compile error ({error}); expected {expected}"
                ));
                continue;
            }
        };
        let expected = normalize::normalize(compiled.program())
            .unwrap_or_else(|error| panic!("{scene}: normalize compiled: {error}"));
        for warning in compiled.warnings() {
            warning_total += 1;
            *warnings_by_command
                .entry(warning.command().to_owned())
                .or_insert(0) += 1;
        }
        let encoded = compiled
            .program()
            .lower()
            .unwrap_or_else(|error| panic!("{scene}: lower compiled: {error}"))
            .encode();
        let round_trip = Script::parse(&encoded)
            .unwrap_or_else(|error| panic!("{scene}: parse round-trip: {error}"));
        let round_trip = round_trip
            .lift()
            .unwrap_or_else(|error| panic!("{scene}: lift round-trip: {error}"));
        let round_trip = normalize::normalize(&round_trip)
            .unwrap_or_else(|error| panic!("{scene}: normalize round-trip: {error}"));
        assert_eq!(round_trip, expected, "{scene}: encode/decode round-trip");
        let slots: Vec<usize> = compiled
            .literal_slots()
            .iter()
            .map(|slot| slot.index)
            .collect();

        let chs_path = chs_dir.join(format!("{}.chs", scene.to_lowercase()));
        let bytes = std::fs::read(&chs_path)
            .unwrap_or_else(|error| panic!("{}: {error}", chs_path.display()));
        let script =
            Script::parse(&bytes).unwrap_or_else(|error| panic!("{scene}: parse chs: {error}"));
        let program = script
            .lift()
            .unwrap_or_else(|error| panic!("{scene}: lift: {error}"));
        let actual = normalize::normalize(&program)
            .unwrap_or_else(|error| panic!("{scene}: normalize steam: {error}"));
        let steam_offsets: Vec<usize> = script
            .instructions()
            .iter()
            .filter(|item| item.opcode() != DEBUG)
            .map(|item| item.offset())
            .collect();
        assert_eq!(
            steam_offsets.len(),
            actual.len(),
            "{scene}: kept instruction count"
        );

        let mut entries: Vec<compare::AuthoredDiff<'_>> = Vec::new();
        let mut owned_source: Vec<Vec<normalize::NormInstruction>> = Vec::new();
        let mut owned_steam: Vec<Vec<normalize::NormInstruction>> = Vec::new();
        for difference in &differences {
            let name = difference
                .get("scene")
                .expect("authored scene")
                .expect_string()
                .expect("scene");
            if name != *scene {
                continue;
            }
            let Value::Array(source_value) = difference
                .get("source_value")
                .expect("authored source_value")
            else {
                panic!("{scene}: authored source_value is not an array");
            };
            let Value::Array(steam_value) =
                difference.get("steam_value").expect("authored steam_value")
            else {
                panic!("{scene}: authored steam_value is not an array");
            };
            owned_source.push(
                source_value
                    .iter()
                    .map(|item| parse_norm_instruction(item, scene).expect("source_value"))
                    .collect(),
            );
            owned_steam.push(
                steam_value
                    .iter()
                    .map(|item| parse_norm_instruction(item, scene).expect("steam_value"))
                    .collect(),
            );
        }
        for difference in differences.iter().filter(|item| {
            item.get("scene")
                .expect("authored scene")
                .expect_string()
                .expect("scene")
                == *scene
        }) {
            let index = entries.len();
            let switch_char = match difference.get("switch_char").expect("switch_char") {
                Value::Number(number) => Some(number as usize),
                Value::Null => None,
                _ => panic!("{scene}: bad switch_char"),
            };
            entries.push(compare::AuthoredDiff {
                source_index: difference.get("source_index").expect("source_index").expect_usize().expect("index"),
                steam_index: difference.get("steam_index").expect("steam_index").expect_usize().expect("index"),
                source_value: &owned_source[index],
                steam_value: &owned_steam[index],
                switch_char,
                steam_offset: difference.get("steam_offset").expect("steam_offset").expect_usize().expect("offset"),
                reviewed: difference.get("reviewed").expect("reviewed") == Value::Bool(true),
                has_evidence: matches!(difference.get("evidence").expect("evidence"), Value::String(text) if !text.is_empty()),
            });
        }

        let result = compare::compare(&compare::SceneInput {
            expected: &expected,
            actual: &actual,
            slots: &slots,
            owners: compiled.owners(),
            switch_chars: &switch_chars,
            steam_offsets: &steam_offsets,
            pool_start: script.string_pool_start(),
            has_problems: false,
            authored: &entries,
        });
        if result.class >= compare::SceneClass::Exact {
            exact += 1;
        }
        if result.class >= compare::SceneClass::Shape {
            shape += 1;
        }
        if result.authored_matching {
            authored_count += 1;
        }
        let Value::String(expected_label) = expected_classes
            .iter()
            .find(|(name, _)| name == scene)
            .map(|(_, class)| class.clone())
            .expect("scene class")
        else {
            panic!("{scene}: class is not a string");
        };
        let expected_class = class_of_label(&expected_label).expect("class label");
        if result.class == expected_class {
        } else if class_rank(&result.class) > class_rank(&expected_class) {
            beats.push(format!(
                "{scene}: Rust {} beats reference {}",
                result.class.label(),
                expected_label
            ));
        } else {
            worse.push(format!(
                "{scene}: Rust {} is worse than reference {} (exact {:?}, shape {:?}, authored failure {:?})",
                result.class.label(),
                expected_label,
                result.exact_divergence,
                result.shape_divergence,
                result.authored_failure
            ));
            if let Some(divergence) = result.exact_divergence {
                let owner = compiled.owners().get(divergence).copied().unwrap_or(-99);
                let statement = if owner >= 0 {
                    statements.get(owner as usize).map(|item| {
                        format!(
                            "{}({}) @{}",
                            item.name(),
                            item.args()
                                .iter()
                                .map(|arg| arg.raw())
                                .collect::<Vec<_>>()
                                .join(","),
                            item.offset()
                        )
                    })
                } else {
                    None
                };
                println!(
                    "divergence {scene} at {divergence}: owner {owner} {statement:?}\n  rust  {:?}\n  steam {:?}",
                    expected.get(divergence),
                    actual.get(divergence)
                );
            }
        }
    }

    println!(
        "compiled {SCENES} scenes: exact={exact} shape={shape} shape+authored={authored_count}"
    );
    println!("unproven zero-emission warnings: {warning_total}");
    for (command, count) in &warnings_by_command {
        println!("warning {command}: {count}");
    }
    println!(
        "per-scene class agreement: {} agree, {} beats, {} worse",
        SCENES - beats.len() - worse.len(),
        beats.len(),
        worse.len()
    );
    for beat in &beats {
        println!("beat: {beat}");
    }
    for failure in &compile_failures {
        println!("compile failure: {failure}");
    }
    assert!(
        worse.is_empty(),
        "scenes worse than the expected classes: {worse:?}"
    );
    assert_eq!(exact, EXPECTED_EXACT, "exact scene count");
    assert_eq!(shape, EXPECTED_SHAPE, "shape scene count");
    assert_eq!(
        authored_count, EXPECTED_AUTHORED,
        "shape+authored scene count"
    );
}
