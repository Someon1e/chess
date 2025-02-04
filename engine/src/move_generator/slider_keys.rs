/// Magic key.
#[derive(Debug, Clone, Copy)]
pub struct Key {
    /// Multiplied with the bit board.
    pub magic: u64,

    /// Offset in the look up table index
    pub offset: u32,
}

/// Rook magic keys.
pub const ROOK_KEYS: [Key; 64] = [
    Key {
        magic: 0x0210_0808_1020_4080,
        offset: 12288,
    },
    Key {
        magic: 0x4020_0010_0008_0020,
        offset: 62607,
    },
    Key {
        magic: 0x0040_1000_4008_0004,
        offset: 64655,
    },
    Key {
        magic: 0x0040_0800_4004_0007,
        offset: 54410,
    },
    Key {
        magic: 0x0040_0400_0200_4001,
        offset: 66703,
    },
    Key {
        magic: 0x0020_0080_0902_1020,
        offset: 48161,
    },
    Key {
        magic: 0x0040_0040_0100_0080,
        offset: 68751,
    },
    Key {
        magic: 0x0080_0020_4500_0080,
        offset: 0,
    },
    Key {
        magic: 0x0000_0844_1104_2202,
        offset: 41723,
    },
    Key {
        magic: 0x0100_1004_0008_0092,
        offset: 90640,
    },
    Key {
        magic: 0x0300_4004_0801_4011,
        offset: 80047,
    },
    Key {
        magic: 0x4400_2004_0002_0020,
        offset: 94758,
    },
    Key {
        magic: 0x0000_2001_0002_0020,
        offset: 95782,
    },
    Key {
        magic: 0x0180_2001_0020_0080,
        offset: 96806,
    },
    Key {
        magic: 0x0080_4000_4000_8001,
        offset: 97830,
    },
    Key {
        magic: 0x0000_2000_4084_0020,
        offset: 50261,
    },
    Key {
        magic: 0x8240_0020_0010_0020,
        offset: 70799,
    },
    Key {
        magic: 0x4084_0010_0008_0150,
        offset: 88520,
    },
    Key {
        magic: 0x0004_0008_0801_0200,
        offset: 78511,
    },
    Key {
        magic: 0x0402_0020_0420_2062,
        offset: 86360,
    },
    Key {
        magic: 0x0001_0020_2002_0001,
        offset: 98854,
    },
    Key {
        magic: 0x0001_0020_0080_2001,
        offset: 99878,
    },
    Key {
        magic: 0x4080_8010_1020_0040,
        offset: 100902,
    },
    Key {
        magic: 0x1020_8020_0040_0020,
        offset: 56460,
    },
    Key {
        magic: 0x1804_1080_0822_4480,
        offset: 43892,
    },
    Key {
        magic: 0x0410_0400_0802_0009,
        offset: 101926,
    },
    Key {
        magic: 0x8400_0480_1002_0010,
        offset: 76943,
    },
    Key {
        magic: 0x0040_0200_2004_0021,
        offset: 102950,
    },
    Key {
        magic: 0x4000_0100_2002_0020,
        offset: 103974,
    },
    Key {
        magic: 0x1108_0100_2000_8020,
        offset: 104998,
    },
    Key {
        magic: 0x1000_0080_2020_0040,
        offset: 106022,
    },
    Key {
        magic: 0x0000_8200_2000_4020,
        offset: 52345,
    },
    Key {
        magic: 0x0440_0010_0020_0020,
        offset: 72847,
    },
    Key {
        magic: 0x8004_0008_0010_0010,
        offset: 107046,
    },
    Key {
        magic: 0x0200_4009_0041_0010,
        offset: 85202,
    },
    Key {
        magic: 0x0000_0402_0020_2002,
        offset: 92708,
    },
    Key {
        magic: 0x0000_0201_0020_2003,
        offset: 91680,
    },
    Key {
        magic: 0x4000_2000_8020_0110,
        offset: 89584,
    },
    Key {
        magic: 0x0200_2000_8020_0C40,
        offset: 87448,
    },
    Key {
        magic: 0x1200_8020_0020_0040,
        offset: 58509,
    },
    Key {
        magic: 0x1021_1042_0400_0801,
        offset: 46060,
    },
    Key {
        magic: 0x0014_0802_0300_0400,
        offset: 83985,
    },
    Key {
        magic: 0x2000_0440_8800_4010,
        offset: 81391,
    },
    Key {
        magic: 0x0800_0400_0200_2020,
        offset: 108070,
    },
    Key {
        magic: 0x0400_0100_0200_2020,
        offset: 109094,
    },
    Key {
        magic: 0x0250_0100_0080_2020,
        offset: 110118,
    },
    Key {
        magic: 0x0920_0080_4000_4001,
        offset: 93733,
    },
    Key {
        magic: 0xBA48_4400_8800_4402,
        offset: 39533,
    },
    Key {
        magic: 0x0440_0010_0020_0020,
        offset: 74895,
    },
    Key {
        magic: 0x0000_0804_0010_0010,
        offset: 111142,
    },
    Key {
        magic: 0x0081_0008_0604_0008,
        offset: 82703,
    },
    Key {
        magic: 0x0800_0400_2002_0020,
        offset: 112166,
    },
    Key {
        magic: 0x4020_2001_0002_0020,
        offset: 113190,
    },
    Key {
        magic: 0x0018_2001_0000_8020,
        offset: 114214,
    },
    Key {
        magic: 0x0204_2000_4000_8020,
        offset: 115238,
    },
    Key {
        magic: 0x0180_8020_0040_0020,
        offset: 60558,
    },
    Key {
        magic: 0x0280_2300_8040_1602,
        offset: 4096,
    },
    Key {
        magic: 0x1080_2080_1009_0041,
        offset: 20349,
    },
    Key {
        magic: 0x6000_0810_2004_8042,
        offset: 16381,
    },
    Key {
        magic: 0x0408_4004_1020_0802,
        offset: 24189,
    },
    Key {
        magic: 0x0000_1004_2001_0802,
        offset: 28029,
    },
    Key {
        magic: 0x0000_0480_0204_1801,
        offset: 31869,
    },
    Key {
        magic: 0xC424_0082_4002_0401,
        offset: 35701,
    },
    Key {
        magic: 0x0000_0902_2884_024A,
        offset: 8192,
    },
];

/// Bishop magic keys.
#[allow(clippy::unreadable_literal)]
pub const BISHOP_KEYS: [Key; 64] = [
    Key {
        magic: 0x0008_0100_4080_0C04,
        offset: 120136,
    },
    Key {
        magic: 0x0000_2040_1040_1000,
        offset: 121555,
    },
    Key {
        magic: 0x1000_8020_4002_8000,
        offset: 121587,
    },
    Key {
        magic: 0x0000_8060_0400_0500,
        offset: 121178,
    },
    Key {
        magic: 0x0040_4402_0A01_2004,
        offset: 120266,
    },
    Key {
        magic: 0x8120_4042_0084_0000,
        offset: 120329,
    },
    Key {
        magic: 0x2000_1041_0400_6130,
        offset: 120644,
    },
    Key {
        magic: 0x1840_1042_0200_80A0,
        offset: 118857,
    },
    Key {
        magic: 0x0000_8042_0040_1020,
        offset: 121619,
    },
    Key {
        magic: 0x0001_0200_8020_0802,
        offset: 121651,
    },
    Key {
        magic: 0x0010_0080_0840_08A1,
        offset: 121683,
    },
    Key {
        magic: 0x0441_0080_4810_8080,
        offset: 121226,
    },
    Key {
        magic: 0x0010_4044_0221_1841,
        offset: 120392,
    },
    Key {
        magic: 0x4120_0041_0080_6408,
        offset: 120455,
    },
    Key {
        magic: 0x0090_4020_2104_0080,
        offset: 120518,
    },
    Key {
        magic: 0x8820_0440_1080_8028,
        offset: 120706,
    },
    Key {
        magic: 0x3000_8004_0100_1020,
        offset: 121715,
    },
    Key {
        magic: 0x1840_8902_0010_1020,
        offset: 121747,
    },
    Key {
        magic: 0x8100_4004_0100_0810,
        offset: 118985,
    },
    Key {
        magic: 0x0000_2002_0080_1100,
        offset: 119113,
    },
    Key {
        magic: 0x0000_2500_8084_0045,
        offset: 118454,
    },
    Key {
        magic: 0x8100_2200_8084_0080,
        offset: 118310,
    },
    Key {
        magic: 0x0880_08C8_5020_8020,
        offset: 121447,
    },
    Key {
        magic: 0x4080_0200_2080_4040,
        offset: 120768,
    },
    Key {
        magic: 0x8000_8040_0082_01A1,
        offset: 121128,
    },
    Key {
        magic: 0x3000_4200_4020_2040,
        offset: 121779,
    },
    Key {
        magic: 0x6002_0100_0080_0880,
        offset: 119241,
    },
    Key {
        magic: 0x0044_0C00_00C1_0200,
        offset: 116262,
    },
    Key {
        magic: 0x4003_0045_2100_4000,
        offset: 116774,
    },
    Key {
        magic: 0x0402_0008_0080_8042,
        offset: 119369,
    },
    Key {
        magic: 0x0000_1004_4020_2140,
        offset: 121483,
    },
    Key {
        magic: 0x2030_0820_8410_1009,
        offset: 121811,
    },
    Key {
        magic: 0x0400_8080_8802_4210,
        offset: 121370,
    },
    Key {
        magic: 0x4000_8080_2500_8020,
        offset: 121843,
    },
    Key {
        magic: 0x0180_4200_4000_8101,
        offset: 119497,
    },
    Key {
        magic: 0x0048_1202_8028_0080,
        offset: 117286,
    },
    Key {
        magic: 0x0001_0204_0002_0102,
        offset: 117798,
    },
    Key {
        magic: 0x2180_2020_8040_4044,
        offset: 119625,
    },
    Key {
        magic: 0x1000_8010_4100_1008,
        offset: 121875,
    },
    Key {
        magic: 0x8240_8004_0488_1010,
        offset: 121907,
    },
    Key {
        magic: 0x0200_4100_8080_005C,
        offset: 121939,
    },
    Key {
        magic: 0x8000_6040_4120_0041,
        offset: 121274,
    },
    Key {
        magic: 0x0040_8880_8080_0040,
        offset: 118725,
    },
    Key {
        magic: 0x1020_0888_4020_0040,
        offset: 118592,
    },
    Key {
        magic: 0x0001_0200_4010_0100,
        offset: 119753,
    },
    Key {
        magic: 0x0000_4044_0020_0040,
        offset: 119881,
    },
    Key {
        magic: 0x5101_0040_0840_0009,
        offset: 121971,
    },
    Key {
        magic: 0x0500_4040_0828_2002,
        offset: 121519,
    },
    Key {
        magic: 0x0200_1041_0400_4008,
        offset: 120830,
    },
    Key {
        magic: 0x8400_0820_8200_2001,
        offset: 120892,
    },
    Key {
        magic: 0x0408_0410_3080_9010,
        offset: 121322,
    },
    Key {
        magic: 0x8800_2050_0840_30A8,
        offset: 120954,
    },
    Key {
        magic: 0x0120_4181_0020_2081,
        offset: 122003,
    },
    Key {
        magic: 0x0002_0040_8100_1000,
        offset: 122035,
    },
    Key {
        magic: 0x4104_0080_2040_0508,
        offset: 122067,
    },
    Key {
        magic: 0x0000_4040_9828_0288,
        offset: 121410,
    },
    Key {
        magic: 0x0020_1100_8200_8081,
        offset: 120009,
    },
    Key {
        magic: 0x0001_0040_2021_0042,
        offset: 120581,
    },
    Key {
        magic: 0x8402_20B0_1011_0062,
        offset: 121076,
    },
    Key {
        magic: 0x4200_4005_0408_4032,
        offset: 121015,
    },
    Key {
        magic: 0x8180_0001_8100_2020,
        offset: 122099,
    },
    Key {
        magic: 0x0020_2102_0040_1020,
        offset: 122131,
    },
    Key {
        magic: 0x0000_0200_4100_0810,
        offset: 122163,
    },
    Key {
        magic: 0x0C88_0200_2040_1002,
        offset: 120202,
    },
];

/// Size of the move lookup table.
pub const SLIDERS_TABLE_SIZE: usize = 122195;
