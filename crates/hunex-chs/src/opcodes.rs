/// One registered opcode: its id, the number of inline dwords that follow it,
/// and a mnemonic where the specs give one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpcodeInfo {
    /// The opcode id.
    pub id: u32,
    /// The number of dword operands that follow the opcode.
    pub operands: usize,
    /// The name from the specs, or `None` when the specs do not name it.
    pub mnemonic: Option<&'static str>,
}

/// The 164 opcode ids (71 base opcodes and 93 commands), sorted by id.
pub const OPCODES: &[OpcodeInfo] = &[
    OpcodeInfo {
        id: 0x000,
        operands: 1,
        mnemonic: Some("PushDword"),
    },
    OpcodeInfo {
        id: 0x001,
        operands: 1,
        mnemonic: Some("PushOffset"),
    },
    OpcodeInfo {
        id: 0x002,
        operands: 1,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x003,
        operands: 1,
        mnemonic: Some("PushString"),
    },
    OpcodeInfo {
        id: 0x008,
        operands: 1,
        mnemonic: Some("ReadMemory"),
    },
    OpcodeInfo {
        id: 0x009,
        operands: 1,
        mnemonic: Some("WriteMemory"),
    },
    OpcodeInfo {
        id: 0x00A,
        operands: 1,
        mnemonic: Some("WriteMemoryArgs"),
    },
    OpcodeInfo {
        id: 0x010,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x011,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x018,
        operands: 0,
        mnemonic: Some("Jmp"),
    },
    OpcodeInfo {
        id: 0x019,
        operands: 1,
        mnemonic: Some("JmpIf"),
    },
    OpcodeInfo {
        id: 0x01A,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x01B,
        operands: 0,
        mnemonic: Some("Return"),
    },
    OpcodeInfo {
        id: 0x01C,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x01D,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x01E,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x01F,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x020,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x021,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x022,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x023,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x024,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x025,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x026,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x027,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x028,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x029,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x02A,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x02B,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x030,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x031,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x032,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x033,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x034,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x035,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x038,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x039,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x03A,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x03F,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x040,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x048,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x07B,
        operands: 3,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x07F,
        operands: 2,
        mnemonic: Some("Debug"),
    },
    OpcodeInfo {
        id: 0x080,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x081,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x082,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x083,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x090,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x092,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x094,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x095,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x0E0,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x0E1,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x0E2,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x0E3,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x0E4,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x0E5,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x0E6,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x0E7,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x0E8,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x0E9,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x0EA,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x0EB,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x0F0,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x0F3,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x0F4,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x0F6,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x0F7,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x0F8,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x0F9,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x0FE,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x10E,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x110,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x111,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x112,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x118,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x120,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x121,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x122,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x128,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x12A,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x12C,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x12F,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x130,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x131,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x138,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x13C,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x140,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x141,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x142,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x14B,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x151,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x152,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x153,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x154,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x158,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x160,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x161,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x162,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x164,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x180,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x184,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x185,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x186,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x190,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x194,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x195,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x196,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x197,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x1A0,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x1A8,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x1C0,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x1C1,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x1C2,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x200,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x201,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x220,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x221,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x222,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x223,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x228,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x240,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x241,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x242,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x243,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x244,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x245,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x246,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x247,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x250,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x251,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x252,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x253,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x254,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x255,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x256,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x257,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x258,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x259,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x25A,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x25B,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x25C,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x25D,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x25E,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x25F,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x260,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x261,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x262,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x263,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x264,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x265,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x270,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x271,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x272,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x273,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x274,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x276,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x278,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x279,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x27A,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x27F,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x300,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x340,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x341,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x380,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x381,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x390,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x391,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x392,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x3A0,
        operands: 0,
        mnemonic: None,
    },
    OpcodeInfo {
        id: 0x3A1,
        operands: 0,
        mnemonic: None,
    },
];

/// Looks up an opcode by id. Returns `None` for an id that is not registered.
pub fn opcode_info(id: u32) -> Option<&'static OpcodeInfo> {
    OPCODES
        .binary_search_by_key(&id, |info| info.id)
        .ok()
        .map(|index| &OPCODES[index])
}
