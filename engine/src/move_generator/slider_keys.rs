/// Magic key.
#[derive(Debug, Clone, Copy)]
pub struct Key {
    /// Multiplied with the bit board.
    pub magic: u64,

    /// The right shift after multiply by `magic`.
    pub shift: u8,

    /// Offset in the look up table index
    pub offset: u32,
}

/// Rook magic keys.
#[allow(clippy::unreadable_literal)]
pub const ROOK_KEYS: [Key; 64] = [
    Key {
        shift: 52,
        magic: 0x2480001022400480,
        offset: 0,
    },
    Key {
        shift: 53,
        magic: 0x40400020001000,
        offset: 4096,
    },
    Key {
        shift: 53,
        magic: 0x100104008200300,
        offset: 6144,
    },
    Key {
        shift: 53,
        magic: 0x100090410002100,
        offset: 8192,
    },
    Key {
        shift: 53,
        magic: 0x8200040810202201,
        offset: 10240,
    },
    Key {
        shift: 53,
        magic: 0x2200840002000810,
        offset: 12288,
    },
    Key {
        shift: 53,
        magic: 0x480250000800200,
        offset: 14336,
    },
    Key {
        shift: 52,
        magic: 0x5800044800C2100,
        offset: 16384,
    },
    Key {
        shift: 53,
        magic: 0x1040802840008003,
        offset: 20480,
    },
    Key {
        shift: 54,
        magic: 0x24804000200080,
        offset: 22528,
    },
    Key {
        shift: 54,
        magic: 0x120100200010C300,
        offset: 23552,
    },
    Key {
        shift: 54,
        magic: 0x1000906601000,
        offset: 24576,
    },
    Key {
        shift: 54,
        magic: 0x80808008004400,
        offset: 25600,
    },
    Key {
        shift: 54,
        magic: 0x20008100C0200,
        offset: 26624,
    },
    Key {
        shift: 54,
        magic: 0x884000810540243,
        offset: 27648,
    },
    Key {
        shift: 53,
        magic: 0x80010000E580,
        offset: 28672,
    },
    Key {
        shift: 53,
        magic: 0x2A08008400290,
        offset: 30720,
    },
    Key {
        shift: 54,
        magic: 0x610040401000600A,
        offset: 32768,
    },
    Key {
        shift: 54,
        magic: 0x8120008080201000,
        offset: 33792,
    },
    Key {
        shift: 54,
        magic: 0x4848008005000,
        offset: 34816,
    },
    Key {
        shift: 54,
        magic: 0x8010005000810,
        offset: 35840,
    },
    Key {
        shift: 54,
        magic: 0x990808004000201,
        offset: 36864,
    },
    Key {
        shift: 54,
        magic: 0x4102940009D60810,
        offset: 37888,
    },
    Key {
        shift: 53,
        magic: 0x8017060024004083,
        offset: 38912,
    },
    Key {
        shift: 53,
        magic: 0x44400180128021,
        offset: 40960,
    },
    Key {
        shift: 54,
        magic: 0x812210100400085,
        offset: 43008,
    },
    Key {
        shift: 54,
        magic: 0x4400A00480100485,
        offset: 44032,
    },
    Key {
        shift: 54,
        magic: 0xA000080280100180,
        offset: 45056,
    },
    Key {
        shift: 54,
        magic: 0x5240080800800,
        offset: 46080,
    },
    Key {
        shift: 54,
        magic: 0x18C500A300040008,
        offset: 47104,
    },
    Key {
        shift: 54,
        magic: 0x5440100400012208,
        offset: 48128,
    },
    Key {
        shift: 53,
        magic: 0x800480004100,
        offset: 49152,
    },
    Key {
        shift: 53,
        magic: 0x4080400020800A80,
        offset: 51200,
    },
    Key {
        shift: 54,
        magic: 0x201002C000C02000,
        offset: 53248,
    },
    Key {
        shift: 54,
        magic: 0x6000C800401001,
        offset: 54272,
    },
    Key {
        shift: 54,
        magic: 0x102100019002100,
        offset: 55296,
    },
    Key {
        shift: 54,
        magic: 0x2008800400800800,
        offset: 56320,
    },
    Key {
        shift: 54,
        magic: 0x90400080C012010,
        offset: 57344,
    },
    Key {
        shift: 54,
        magic: 0x2000181024002706,
        offset: 58368,
    },
    Key {
        shift: 53,
        magic: 0x1004592000104,
        offset: 59392,
    },
    Key {
        shift: 53,
        magic: 0x4500400080388000,
        offset: 61440,
    },
    Key {
        shift: 54,
        magic: 0x1000500820004004,
        offset: 63488,
    },
    Key {
        shift: 54,
        magic: 0x120008010008020,
        offset: 64512,
    },
    Key {
        shift: 54,
        magic: 0x10010021000B,
        offset: 65536,
    },
    Key {
        shift: 54,
        magic: 0x4888000500090010,
        offset: 66560,
    },
    Key {
        shift: 54,
        magic: 0x2000824020050,
        offset: 67584,
    },
    Key {
        shift: 54,
        magic: 0x30810140006,
        offset: 68608,
    },
    Key {
        shift: 53,
        magic: 0x40E1000080C10002,
        offset: 69632,
    },
    Key {
        shift: 54,
        magic: 0x48FFFE99FECFAA00,
        offset: 71680,
    },
    Key {
        shift: 55,
        magic: 0x48FFFE99FECFAA00,
        offset: 72704,
    },
    Key {
        shift: 55,
        magic: 0x497FFFADFF9C2E00,
        offset: 73216,
    },
    Key {
        shift: 55,
        magic: 0x613FFFDDFFCE9200,
        offset: 73728,
    },
    Key {
        shift: 55,
        magic: 0xFFFFFFE9FFE7CE00,
        offset: 74240,
    },
    Key {
        shift: 55,
        magic: 0xFFFFFFF5FFF3E600,
        offset: 74752,
    },
    Key {
        shift: 55,
        magic: 0x3FF95E5E6A4C0,
        offset: 75264,
    },
    Key {
        shift: 54,
        magic: 0x510FFFF5F63C96A0,
        offset: 75776,
    },
    Key {
        shift: 53,
        magic: 0xEBFFFFB9FF9FC526,
        offset: 76800,
    },
    Key {
        shift: 54,
        magic: 0x61FFFEDDFEEDAEAE,
        offset: 78848,
    },
    Key {
        shift: 54,
        magic: 0x53BFFFEDFFDEB1A2,
        offset: 79872,
    },
    Key {
        shift: 54,
        magic: 0x127FFFB9FFDFB5F6,
        offset: 80896,
    },
    Key {
        shift: 54,
        magic: 0x411FFFDDFFDBF4D6,
        offset: 81920,
    },
    Key {
        shift: 53,
        magic: 0x1150024000A0811,
        offset: 82944,
    },
    Key {
        shift: 54,
        magic: 0x3FFEF27EEBE74,
        offset: 84992,
    },
    Key {
        shift: 53,
        magic: 0x7645FFFECBFEA79E,
        offset: 86016,
    },
];

/// Size of the rook move lookup table.
pub const ROOK_TABLE_SIZE: usize = 88064;

/// Bishop magic keys.
#[allow(clippy::unreadable_literal)]
pub const BISHOP_KEYS: [Key; 64] = [
    Key {
        shift: 59,
        magic: 0xFFEDF9FD7CFCFFFF,
        offset: 0,
    },
    Key {
        shift: 60,
        magic: 0xFC0962854A77F576,
        offset: 32,
    },
    Key {
        shift: 59,
        magic: 0x920420C2010080,
        offset: 48,
    },
    Key {
        shift: 59,
        magic: 0x2004410020240409,
        offset: 80,
    },
    Key {
        shift: 59,
        magic: 0x85104000000100,
        offset: 112,
    },
    Key {
        shift: 59,
        magic: 0x424044040C210,
        offset: 144,
    },
    Key {
        shift: 60,
        magic: 0xFC0A66C64A7EF576,
        offset: 176,
    },
    Key {
        shift: 59,
        magic: 0x7FFDFDFCBD79FFFF,
        offset: 192,
    },
    Key {
        shift: 60,
        magic: 0xFC0846A64A34FFF6,
        offset: 224,
    },
    Key {
        shift: 60,
        magic: 0xFC087A874A3CF7F6,
        offset: 240,
    },
    Key {
        shift: 59,
        magic: 0x28C0090802028200,
        offset: 256,
    },
    Key {
        shift: 59,
        magic: 0x4208080600400000,
        offset: 288,
    },
    Key {
        shift: 59,
        magic: 0x120A10005040,
        offset: 320,
    },
    Key {
        shift: 59,
        magic: 0x126643004100001,
        offset: 352,
    },
    Key {
        shift: 60,
        magic: 0xFC0864AE59B4FF76,
        offset: 384,
    },
    Key {
        shift: 60,
        magic: 0x3C0860AF4B35FF76,
        offset: 400,
    },
    Key {
        shift: 60,
        magic: 0x73C01AF56CF4CFFB,
        offset: 416,
    },
    Key {
        shift: 60,
        magic: 0x41A01CFAD64AAFFC,
        offset: 432,
    },
    Key {
        shift: 57,
        magic: 0x4008200220A00,
        offset: 448,
    },
    Key {
        shift: 57,
        magic: 0x4008004220604040,
        offset: 576,
    },
    Key {
        shift: 57,
        magic: 0x2000400940100,
        offset: 704,
    },
    Key {
        shift: 57,
        magic: 0x2020108050402,
        offset: 832,
    },
    Key {
        shift: 60,
        magic: 0x7C0C028F5B34FF76,
        offset: 960,
    },
    Key {
        shift: 60,
        magic: 0xFC0A028E5AB4DF76,
        offset: 976,
    },
    Key {
        shift: 59,
        magic: 0x20220044080221,
        offset: 992,
    },
    Key {
        shift: 59,
        magic: 0x48084200604C4110,
        offset: 1024,
    },
    Key {
        shift: 57,
        magic: 0x4144280C10088022,
        offset: 1056,
    },
    Key {
        shift: 55,
        magic: 0x5304080000A20040,
        offset: 1184,
    },
    Key {
        shift: 55,
        magic: 0x1026040002008200,
        offset: 1696,
    },
    Key {
        shift: 57,
        magic: 0x8450A0041080120,
        offset: 2208,
    },
    Key {
        shift: 59,
        magic: 0x18910C0001148800,
        offset: 2336,
    },
    Key {
        shift: 59,
        magic: 0x3440408104420821,
        offset: 2368,
    },
    Key {
        shift: 59,
        magic: 0x81080848405000,
        offset: 2400,
    },
    Key {
        shift: 59,
        magic: 0x1010800901042,
        offset: 2432,
    },
    Key {
        shift: 57,
        magic: 0x240C004400480124,
        offset: 2464,
    },
    Key {
        shift: 55,
        magic: 0x1900110800040040,
        offset: 2592,
    },
    Key {
        shift: 55,
        magic: 0x40401808208A0020,
        offset: 3104,
    },
    Key {
        shift: 57,
        magic: 0x1002040440080802,
        offset: 3616,
    },
    Key {
        shift: 59,
        magic: 0x4081440080430800,
        offset: 3744,
    },
    Key {
        shift: 59,
        magic: 0x8004141180086080,
        offset: 3776,
    },
    Key {
        shift: 60,
        magic: 0xDCEFD9B54BFCC09F,
        offset: 3808,
    },
    Key {
        shift: 60,
        magic: 0xF95FFA765AFD602B,
        offset: 3824,
    },
    Key {
        shift: 57,
        magic: 0x501040126004402,
        offset: 3840,
    },
    Key {
        shift: 57,
        magic: 0x82202093006800,
        offset: 3968,
    },
    Key {
        shift: 57,
        magic: 0x8C82082104000044,
        offset: 4096,
    },
    Key {
        shift: 57,
        magic: 0x8C0500412612040,
        offset: 4224,
    },
    Key {
        shift: 60,
        magic: 0x43FF9A5CF4CA0C01,
        offset: 4352,
    },
    Key {
        shift: 60,
        magic: 0x4BFFCD8E7C587601,
        offset: 4368,
    },
    Key {
        shift: 60,
        magic: 0xFC0FF2865334F576,
        offset: 4384,
    },
    Key {
        shift: 60,
        magic: 0xFC0BF6CE5924F576,
        offset: 4400,
    },
    Key {
        shift: 59,
        magic: 0x10020042080001,
        offset: 4416,
    },
    Key {
        shift: 59,
        magic: 0x40246080028,
        offset: 4448,
    },
    Key {
        shift: 59,
        magic: 0x1800021020222000,
        offset: 4480,
    },
    Key {
        shift: 59,
        magic: 0x1008081A18020008,
        offset: 4512,
    },
    Key {
        shift: 60,
        magic: 0xC3FFB7DC36CA8C89,
        offset: 4544,
    },
    Key {
        shift: 60,
        magic: 0xC3FF8A54F4CA2C89,
        offset: 4560,
    },
    Key {
        shift: 59,
        magic: 0xFFFFFCFCFD79EDFF,
        offset: 4576,
    },
    Key {
        shift: 60,
        magic: 0xFC0863FCCB147576,
        offset: 4608,
    },
    Key {
        shift: 59,
        magic: 0x4084008881,
        offset: 4624,
    },
    Key {
        shift: 59,
        magic: 0x58200043940414,
        offset: 4656,
    },
    Key {
        shift: 59,
        magic: 0x20000029502C00,
        offset: 4688,
    },
    Key {
        shift: 59,
        magic: 0x408001410C041420,
        offset: 4720,
    },
    Key {
        shift: 60,
        magic: 0xFC087E8E4BB2F736,
        offset: 4752,
    },
    Key {
        shift: 59,
        magic: 0x43FF9E4EF4CA2C89,
        offset: 4768,
    },
];

/// Size of the bishop move lookup table.
pub const BISHOP_TABLE_SIZE: usize = 4800;
