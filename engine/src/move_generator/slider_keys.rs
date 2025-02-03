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
        magic: 0x1080_0040_0022_8050,
        offset: 0,
    },
    Key {
        magic: 0x0020_0008_0010_0020,
        offset: 73239,
    },
    Key {
        magic: 0x0040_0800_4011_0005,
        offset: 60790,
    },
    Key {
        magic: 0x0040_0800_4004_0002,
        offset: 75287,
    },
    Key {
        magic: 0x0040_0200_0114_0040,
        offset: 49619,
    },
    Key {
        magic: 0x0020_2000_8001_0208,
        offset: 67057,
    },
    Key {
        magic: 0x0040_0080_0104_0040,
        offset: 69125,
    },
    Key {
        magic: 0x0080_0100_0070_4080,
        offset: 4096,
    },
    Key {
        magic: 0x1800_4404_0100_2200,
        offset: 51939,
    },
    Key {
        magic: 0x0000_1000_1401_0801,
        offset: 107434,
    },
    Key {
        magic: 0x0800_0804_1082_4008,
        offset: 81431,
    },
    Key {
        magic: 0x0000_2004_4021_020C,
        offset: 121974,
    },
    Key {
        magic: 0x2480_2002_0020_0100,
        offset: 130756,
    },
    Key {
        magic: 0x0000_2020_0080_0102,
        offset: 129728,
    },
    Key {
        magic: 0x0801_1010_0808_4002,
        offset: 116598,
    },
    Key {
        magic: 0x3000_2002_2000_4090,
        offset: 56414,
    },
    Key {
        magic: 0x0040_0020_0010_0020,
        offset: 77335,
    },
    Key {
        magic: 0x0820_4004_0210_0800,
        offset: 85501,
    },
    Key {
        magic: 0x0200_2042_0041_0004,
        offset: 102323,
    },
    Key {
        magic: 0x3001_8008_0400_C008,
        offset: 89512,
    },
    Key {
        magic: 0x0432_0020_0201_0020,
        offset: 110554,
    },
    Key {
        magic: 0x4022_0020_0100_0080,
        offset: 87528,
    },
    Key {
        magic: 0x0000_0041_0400_8001,
        offset: 96944,
    },
    Key {
        magic: 0x000A_0110_0400_2008,
        offset: 44791,
    },
    Key {
        magic: 0x0520_0800_1001_5000,
        offset: 47295,
    },
    Key {
        magic: 0x0509_0400_1009_1100,
        offset: 91402,
    },
    Key {
        magic: 0x0080_8400_1006_0010,
        offset: 95119,
    },
    Key {
        magic: 0x8201_0200_2020_0400,
        offset: 123164,
    },
    Key {
        magic: 0x0000_0100_2002_1020,
        offset: 128688,
    },
    Key {
        magic: 0x0010_0100_2020_0080,
        offset: 131780,
    },
    Key {
        magic: 0x0000_2001_1004_4008,
        offset: 113626,
    },
    Key {
        magic: 0x0000_2000_2000_4090,
        offset: 58610,
    },
    Key {
        magic: 0x0410_2041_0008_0408,
        offset: 64987,
    },
    Key {
        magic: 0x8100_8008_0012_2100,
        offset: 71189,
    },
    Key {
        magic: 0x0000_4108_0040_0410,
        offset: 118046,
    },
    Key {
        magic: 0x0082_0081_0008_0008,
        offset: 112090,
    },
    Key {
        magic: 0x0401_4082_0040_0104,
        offset: 105770,
    },
    Key {
        magic: 0x8000_2008_8020_0102,
        offset: 127644,
    },
    Key {
        magic: 0x1001_0004_0088_2001,
        offset: 104068,
    },
    Key {
        magic: 0x8810_0488_0088_0201,
        offset: 54190,
    },
    Key {
        magic: 0x0240_0020_1000_2008,
        offset: 42231,
    },
    Key {
        magic: 0x0002_0100_8040_8400,
        offset: 120702,
    },
    Key {
        magic: 0x9200_0440_8800_4010,
        offset: 119390,
    },
    Key {
        magic: 0x0100_4082_0010_0100,
        offset: 83468,
    },
    Key {
        magic: 0x0024_0020_1100_2002,
        offset: 109002,
    },
    Key {
        magic: 0x0000_4001_0000_C002,
        offset: 100547,
    },
    Key {
        magic: 0x0200_2000_4001_1009,
        offset: 115118,
    },
    Key {
        magic: 0x6000_080A_2001_1004,
        offset: 39647,
    },
    Key {
        magic: 0x0040_0010_0020_0020,
        offset: 79383,
    },
    Key {
        magic: 0x0012_0008_0204_0008,
        offset: 125468,
    },
    Key {
        magic: 0x4000_8810_0201_0008,
        offset: 93291,
    },
    Key {
        magic: 0x2080_0480_2002_0020,
        offset: 126556,
    },
    Key {
        magic: 0x1000_2001_0002_8020,
        offset: 124316,
    },
    Key {
        magic: 0x0000_2000_8001_0020,
        offset: 132804,
    },
    Key {
        magic: 0x0000_0080_0100_0040,
        offset: 98755,
    },
    Key {
        magic: 0x2000_0040_8420_0020,
        offset: 62902,
    },
    Key {
        magic: 0x08C0_2102_1203_C082,
        offset: 8192,
    },
    Key {
        magic: 0x1801_A100_1089_8041,
        offset: 20352,
    },
    Key {
        magic: 0x0000_0820_8040_0412,
        offset: 16384,
    },
    Key {
        magic: 0x0000_0440_2010_0802,
        offset: 24288,
    },
    Key {
        magic: 0x0000_0104_2008_1002,
        offset: 28128,
    },
    Key {
        magic: 0x1480_0090_0104_0802,
        offset: 31968,
    },
    Key {
        magic: 0x0010_0202_4000_8401,
        offset: 35808,
    },
    Key {
        magic: 0x0000_1400_4088_6102,
        offset: 12288,
    },
];

/// Bishop magic keys.
#[allow(clippy::unreadable_literal)]
pub const BISHOP_KEYS: [Key; 64] = [
    Key {
        magic: 0x0080_8180_1040_8020,
        offset: 139187,
    },
    Key {
        magic: 0x4402_A100_2028_0600,
        offset: 140250,
    },
    Key {
        magic: 0x6080_4010_8410_0000,
        offset: 140507,
    },
    Key {
        magic: 0x2200_8090_0202_0044,
        offset: 140016,
    },
    Key {
        magic: 0x2C00_4402_2000_0880,
        offset: 139517,
    },
    Key {
        magic: 0x6400_4200_8091_0110,
        offset: 139027,
    },
    Key {
        magic: 0x0400_1101_0100_8010,
        offset: 139580,
    },
    Key {
        magic: 0x0020_2021_0400_8050,
        offset: 138212,
    },
    Key {
        magic: 0x8080_0100_4100_4004,
        offset: 140982,
    },
    Key {
        magic: 0x8108_0042_0008_4808,
        offset: 140136,
    },
    Key {
        magic: 0x4040_8100_100C_1011,
        offset: 140914,
    },
    Key {
        magic: 0x0010_0100_4404_4411,
        offset: 139894,
    },
    Key {
        magic: 0x0001_0044_0200_0008,
        offset: 139643,
    },
    Key {
        magic: 0x2048_0022_8200_8040,
        offset: 139107,
    },
    Key {
        magic: 0x8008_1010_4104_0058,
        offset: 139832,
    },
    Key {
        magic: 0x0A08_0008_0402_0040,
        offset: 137213,
    },
    Key {
        magic: 0x0202_0101_0200_600A,
        offset: 140878,
    },
    Key {
        magic: 0x0002_3201_0100_1010,
        offset: 140556,
    },
    Key {
        magic: 0x0881_0002_0200_1812,
        offset: 137669,
    },
    Key {
        magic: 0x4200_2002_0820_0400,
        offset: 138083,
    },
    Key {
        magic: 0x1C20_8020_1004_0080,
        offset: 136070,
    },
    Key {
        magic: 0x2020_1000_2010_0890,
        offset: 135876,
    },
    Key {
        magic: 0x8020_0410_1121_00E0,
        offset: 139259,
    },
    Key {
        magic: 0x0240_4010_1010_8020,
        offset: 139706,
    },
    Key {
        magic: 0x0802_1000_0200_8022,
        offset: 138649,
    },
    Key {
        magic: 0x0002_0060_0080_8010,
        offset: 140357,
    },
    Key {
        magic: 0x0902_0100_0200_9004,
        offset: 137381,
    },
    Key {
        magic: 0x0004_0400_0041_0200,
        offset: 133828,
    },
    Key {
        magic: 0x0001_0040_0401_4041,
        offset: 134340,
    },
    Key {
        magic: 0x0100_2020_0040_1001,
        offset: 136262,
    },
    Key {
        magic: 0x0000_1020_0080_8010,
        offset: 140076,
    },
    Key {
        magic: 0x0010_0844_4008_2080,
        offset: 139324,
    },
    Key {
        magic: 0x0001_0101_0000_8064,
        offset: 140948,
    },
    Key {
        magic: 0x0210_4040_2042_4101,
        offset: 140604,
    },
    Key {
        magic: 0x0001_8100_8001_0108,
        offset: 136454,
    },
    Key {
        magic: 0xA106_2200_8048_0080,
        offset: 134852,
    },
    Key {
        magic: 0x1000_4080_6002_0200,
        offset: 135364,
    },
    Key {
        magic: 0x000C_0090_5000_2084,
        offset: 137029,
    },
    Key {
        magic: 0x0000_2080_2040_1008,
        offset: 141014,
    },
    Key {
        magic: 0x11C1_0008_0200_0821,
        offset: 140652,
    },
    Key {
        magic: 0x0000_2402_0081_8209,
        offset: 138744,
    },
    Key {
        magic: 0x0000_4042_0020_0060,
        offset: 139955,
    },
    Key {
        magic: 0x2018_0840_0800_0089,
        offset: 136646,
    },
    Key {
        magic: 0x8400_9810_8048_4080,
        offset: 137809,
    },
    Key {
        magic: 0x4850_8400_8080_2024,
        offset: 137947,
    },
    Key {
        magic: 0x0011_0200_4049_2010,
        offset: 137525,
    },
    Key {
        magic: 0x8004_0030_1010_0022,
        offset: 140407,
    },
    Key {
        magic: 0x8000_6088_2021_1408,
        offset: 140305,
    },
    Key {
        magic: 0x2010_4044_0080_A00C,
        offset: 138457,
    },
    Key {
        magic: 0x8040_0821_0100_4B11,
        offset: 139769,
    },
    Key {
        magic: 0x0020_8020_2200_4131,
        offset: 138839,
    },
    Key {
        magic: 0x0180_0318_0840_4210,
        offset: 139389,
    },
    Key {
        magic: 0x0003_2804_0040_1020,
        offset: 140194,
    },
    Key {
        magic: 0x0120_0040_C190_1814,
        offset: 140700,
    },
    Key {
        magic: 0x0906_0040_A008_1201,
        offset: 140796,
    },
    Key {
        magic: 0x1004_0010_4008_080C,
        offset: 140748,
    },
    Key {
        magic: 0x3081_0080_2101_0088,
        offset: 136838,
    },
    Key {
        magic: 0x1800_0420_1202_0022,
        offset: 138339,
    },
    Key {
        magic: 0x0004_5044_2022_0044,
        offset: 138933,
    },
    Key {
        magic: 0x0210_0000_4008_4040,
        offset: 139453,
    },
    Key {
        magic: 0x0C05_8000_0100_2020,
        offset: 141046,
    },
    Key {
        magic: 0x0081_9802_8101_0009,
        offset: 140838,
    },
    Key {
        magic: 0x0100_0104_4080_0808,
        offset: 140457,
    },
    Key {
        magic: 0x8004_0100_1021_0004,
        offset: 138553,
    },
];

/// Size of the move lookup table.
pub const SLIDERS_TABLE_SIZE: usize = 141078;
