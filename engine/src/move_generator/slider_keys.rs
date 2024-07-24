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
pub const ROOK_KEYS: [Key; 64] = [
    Key {
        shift: 52,
        magic: 0x2480_0010_2240_0480,
        offset: 0,
    },
    Key {
        shift: 53,
        magic: 0x0040_4000_2000_1000,
        offset: 4096,
    },
    Key {
        shift: 53,
        magic: 0x0100_1040_0820_0300,
        offset: 6144,
    },
    Key {
        shift: 53,
        magic: 0x0100_0904_1000_2100,
        offset: 8192,
    },
    Key {
        shift: 53,
        magic: 0x8200_0408_1020_2201,
        offset: 10240,
    },
    Key {
        shift: 53,
        magic: 0x2200_8400_0200_0810,
        offset: 12288,
    },
    Key {
        shift: 53,
        magic: 0x0480_2500_0080_0200,
        offset: 14336,
    },
    Key {
        shift: 52,
        magic: 0x0580_0044_800C_2100,
        offset: 16384,
    },
    Key {
        shift: 53,
        magic: 0x1040_8028_4000_8003,
        offset: 20480,
    },
    Key {
        shift: 54,
        magic: 0x0024_8040_0020_0080,
        offset: 22528,
    },
    Key {
        shift: 54,
        magic: 0x1201_0020_0010_C300,
        offset: 23552,
    },
    Key {
        shift: 54,
        magic: 0x0001_0009_0660_1000,
        offset: 24576,
    },
    Key {
        shift: 54,
        magic: 0x0080_8080_0800_4400,
        offset: 25600,
    },
    Key {
        shift: 54,
        magic: 0x0002_0008_100C_0200,
        offset: 26624,
    },
    Key {
        shift: 54,
        magic: 0x0884_0008_1054_0243,
        offset: 27648,
    },
    Key {
        shift: 53,
        magic: 0x0000_8001_0000_E580,
        offset: 28672,
    },
    Key {
        shift: 53,
        magic: 0x0002_A080_0840_0290,
        offset: 30720,
    },
    Key {
        shift: 54,
        magic: 0x6100_4040_1000_600A,
        offset: 32768,
    },
    Key {
        shift: 54,
        magic: 0x8120_0080_8020_1000,
        offset: 33792,
    },
    Key {
        shift: 54,
        magic: 0x0004_8480_0800_5000,
        offset: 34816,
    },
    Key {
        shift: 54,
        magic: 0x0008_0100_0500_0810,
        offset: 35840,
    },
    Key {
        shift: 54,
        magic: 0x0990_8080_0400_0201,
        offset: 36864,
    },
    Key {
        shift: 54,
        magic: 0x4102_9400_09D6_0810,
        offset: 37888,
    },
    Key {
        shift: 53,
        magic: 0x8017_0600_2400_4083,
        offset: 38912,
    },
    Key {
        shift: 53,
        magic: 0x0044_4001_8012_8021,
        offset: 40960,
    },
    Key {
        shift: 54,
        magic: 0x0812_2101_0040_0085,
        offset: 43008,
    },
    Key {
        shift: 54,
        magic: 0x4400_A004_8010_0485,
        offset: 44032,
    },
    Key {
        shift: 54,
        magic: 0xA000_0802_8010_0180,
        offset: 45056,
    },
    Key {
        shift: 54,
        magic: 0x0005_2400_8080_0800,
        offset: 46080,
    },
    Key {
        shift: 54,
        magic: 0x18C5_00A3_0004_0008,
        offset: 47104,
    },
    Key {
        shift: 54,
        magic: 0x5440_1004_0001_2208,
        offset: 48128,
    },
    Key {
        shift: 53,
        magic: 0x0000_8004_8000_4100,
        offset: 49152,
    },
    Key {
        shift: 53,
        magic: 0x4080_4000_2080_0A80,
        offset: 51200,
    },
    Key {
        shift: 54,
        magic: 0x2010_02C0_00C0_2000,
        offset: 53248,
    },
    Key {
        shift: 54,
        magic: 0x0060_00C8_0040_1001,
        offset: 54272,
    },
    Key {
        shift: 54,
        magic: 0x0102_1000_1900_2100,
        offset: 55296,
    },
    Key {
        shift: 54,
        magic: 0x2008_8004_0080_0800,
        offset: 56320,
    },
    Key {
        shift: 54,
        magic: 0x0904_0008_0C01_2010,
        offset: 57344,
    },
    Key {
        shift: 54,
        magic: 0x2000_1810_2400_2706,
        offset: 58368,
    },
    Key {
        shift: 53,
        magic: 0x0001_0045_9200_0104,
        offset: 59392,
    },
    Key {
        shift: 53,
        magic: 0x4500_4000_8038_8000,
        offset: 61440,
    },
    Key {
        shift: 54,
        magic: 0x1000_5008_2000_4004,
        offset: 63488,
    },
    Key {
        shift: 54,
        magic: 0x0120_0080_1000_8020,
        offset: 64512,
    },
    Key {
        shift: 54,
        magic: 0x0000_1001_0021_000B,
        offset: 65536,
    },
    Key {
        shift: 54,
        magic: 0x4888_0005_0009_0010,
        offset: 66560,
    },
    Key {
        shift: 54,
        magic: 0x0002_0008_2402_0050,
        offset: 67584,
    },
    Key {
        shift: 54,
        magic: 0x0000_0308_1014_0006,
        offset: 68608,
    },
    Key {
        shift: 53,
        magic: 0x40E1_0000_80C1_0002,
        offset: 69632,
    },
    Key {
        shift: 54,
        magic: 0x48FF_FE99_FECF_AA00,
        offset: 71680,
    },
    Key {
        shift: 55,
        magic: 0x48FF_FE99_FECF_AA00,
        offset: 72704,
    },
    Key {
        shift: 55,
        magic: 0x497F_FFAD_FF9C_2E00,
        offset: 73216,
    },
    Key {
        shift: 55,
        magic: 0x613F_FFDD_FFCE_9200,
        offset: 73728,
    },
    Key {
        shift: 55,
        magic: 0xFFFF_FFE9_FFE7_CE00,
        offset: 74240,
    },
    Key {
        shift: 55,
        magic: 0xFFFF_FFF5_FFF3_E600,
        offset: 74752,
    },
    Key {
        shift: 55,
        magic: 0x0003_FF95_E5E6_A4C0,
        offset: 75264,
    },
    Key {
        shift: 54,
        magic: 0x510F_FFF5_F63C_96A0,
        offset: 75776,
    },
    Key {
        shift: 53,
        magic: 0xEBFF_FFB9_FF9F_C526,
        offset: 76800,
    },
    Key {
        shift: 54,
        magic: 0x61FF_FEDD_FEED_AEAE,
        offset: 78848,
    },
    Key {
        shift: 54,
        magic: 0x53BF_FFED_FFDE_B1A2,
        offset: 79872,
    },
    Key {
        shift: 54,
        magic: 0x127F_FFB9_FFDF_B5F6,
        offset: 80896,
    },
    Key {
        shift: 54,
        magic: 0x411F_FFDD_FFDB_F4D6,
        offset: 81920,
    },
    Key {
        shift: 53,
        magic: 0x0115_0024_000A_0811,
        offset: 82944,
    },
    Key {
        shift: 54,
        magic: 0x0003_FFEF_27EE_BE74,
        offset: 84992,
    },
    Key {
        shift: 53,
        magic: 0x7645_FFFE_CBFE_A79E,
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
        magic: 0xFFED_F9FD_7CFC_FFFF,
        offset: 0,
    },
    Key {
        shift: 60,
        magic: 0xFC09_6285_4A77_F576,
        offset: 32,
    },
    Key {
        shift: 59,
        magic: 0x0092_0420_C201_0080,
        offset: 48,
    },
    Key {
        shift: 59,
        magic: 0x2004_4100_2024_0409,
        offset: 80,
    },
    Key {
        shift: 59,
        magic: 0x0085_1040_0000_0100,
        offset: 112,
    },
    Key {
        shift: 59,
        magic: 0x0004_2404_4040_C210,
        offset: 144,
    },
    Key {
        shift: 60,
        magic: 0xFC0A_66C6_4A7E_F576,
        offset: 176,
    },
    Key {
        shift: 59,
        magic: 0x7FFD_FDFC_BD79_FFFF,
        offset: 192,
    },
    Key {
        shift: 60,
        magic: 0xFC08_46A6_4A34_FFF6,
        offset: 224,
    },
    Key {
        shift: 60,
        magic: 0xFC08_7A87_4A3C_F7F6,
        offset: 240,
    },
    Key {
        shift: 59,
        magic: 0x28C0_0908_0202_8200,
        offset: 256,
    },
    Key {
        shift: 59,
        magic: 0x4208_0806_0040_0000,
        offset: 288,
    },
    Key {
        shift: 59,
        magic: 0x0000_120A_1000_5040,
        offset: 320,
    },
    Key {
        shift: 59,
        magic: 0x0126_6430_0410_0001,
        offset: 352,
    },
    Key {
        shift: 60,
        magic: 0xFC08_64AE_59B4_FF76,
        offset: 384,
    },
    Key {
        shift: 60,
        magic: 0x3C08_60AF_4B35_FF76,
        offset: 400,
    },
    Key {
        shift: 60,
        magic: 0x73C0_1AF5_6CF4_CFFB,
        offset: 416,
    },
    Key {
        shift: 60,
        magic: 0x41A0_1CFA_D64A_AFFC,
        offset: 432,
    },
    Key {
        shift: 57,
        magic: 0x0004_0082_0022_0A00,
        offset: 448,
    },
    Key {
        shift: 57,
        magic: 0x4008_0042_2060_4040,
        offset: 576,
    },
    Key {
        shift: 57,
        magic: 0x0002_0004_0094_0100,
        offset: 704,
    },
    Key {
        shift: 57,
        magic: 0x0002_0201_0805_0402,
        offset: 832,
    },
    Key {
        shift: 60,
        magic: 0x7C0C_028F_5B34_FF76,
        offset: 960,
    },
    Key {
        shift: 60,
        magic: 0xFC0A_028E_5AB4_DF76,
        offset: 976,
    },
    Key {
        shift: 59,
        magic: 0x0020_2200_4408_0221,
        offset: 992,
    },
    Key {
        shift: 59,
        magic: 0x4808_4200_604C_4110,
        offset: 1024,
    },
    Key {
        shift: 57,
        magic: 0x4144_280C_1008_8022,
        offset: 1056,
    },
    Key {
        shift: 55,
        magic: 0x5304_0800_00A2_0040,
        offset: 1184,
    },
    Key {
        shift: 55,
        magic: 0x1026_0400_0200_8200,
        offset: 1696,
    },
    Key {
        shift: 57,
        magic: 0x0845_0A00_4108_0120,
        offset: 2208,
    },
    Key {
        shift: 59,
        magic: 0x1891_0C00_0114_8800,
        offset: 2336,
    },
    Key {
        shift: 59,
        magic: 0x3440_4081_0442_0821,
        offset: 2368,
    },
    Key {
        shift: 59,
        magic: 0x0081_0808_4840_5000,
        offset: 2400,
    },
    Key {
        shift: 59,
        magic: 0x0001_0108_0090_1042,
        offset: 2432,
    },
    Key {
        shift: 57,
        magic: 0x240C_0044_0048_0124,
        offset: 2464,
    },
    Key {
        shift: 55,
        magic: 0x1900_1108_0004_0040,
        offset: 2592,
    },
    Key {
        shift: 55,
        magic: 0x4040_1808_208A_0020,
        offset: 3104,
    },
    Key {
        shift: 57,
        magic: 0x1002_0404_4008_0802,
        offset: 3616,
    },
    Key {
        shift: 59,
        magic: 0x4081_4400_8043_0800,
        offset: 3744,
    },
    Key {
        shift: 59,
        magic: 0x8004_1411_8008_6080,
        offset: 3776,
    },
    Key {
        shift: 60,
        magic: 0xDCEF_D9B5_4BFC_C09F,
        offset: 3808,
    },
    Key {
        shift: 60,
        magic: 0xF95F_FA76_5AFD_602B,
        offset: 3824,
    },
    Key {
        shift: 57,
        magic: 0x0501_0401_2600_4402,
        offset: 3840,
    },
    Key {
        shift: 57,
        magic: 0x0082_2020_9300_6800,
        offset: 3968,
    },
    Key {
        shift: 57,
        magic: 0x8C82_0821_0400_0044,
        offset: 4096,
    },
    Key {
        shift: 57,
        magic: 0x08C0_5004_1261_2040,
        offset: 4224,
    },
    Key {
        shift: 60,
        magic: 0x43FF_9A5C_F4CA_0C01,
        offset: 4352,
    },
    Key {
        shift: 60,
        magic: 0x4BFF_CD8E_7C58_7601,
        offset: 4368,
    },
    Key {
        shift: 60,
        magic: 0xFC0F_F286_5334_F576,
        offset: 4384,
    },
    Key {
        shift: 60,
        magic: 0xFC0B_F6CE_5924_F576,
        offset: 4400,
    },
    Key {
        shift: 59,
        magic: 0x0010_0200_4208_0001,
        offset: 4416,
    },
    Key {
        shift: 59,
        magic: 0x0000_0402_4608_0028,
        offset: 4448,
    },
    Key {
        shift: 59,
        magic: 0x1800_0210_2022_2000,
        offset: 4480,
    },
    Key {
        shift: 59,
        magic: 0x1008_081A_1802_0008,
        offset: 4512,
    },
    Key {
        shift: 60,
        magic: 0xC3FF_B7DC_36CA_8C89,
        offset: 4544,
    },
    Key {
        shift: 60,
        magic: 0xC3FF_8A54_F4CA_2C89,
        offset: 4560,
    },
    Key {
        shift: 59,
        magic: 0xFFFF_FCFC_FD79_EDFF,
        offset: 4576,
    },
    Key {
        shift: 60,
        magic: 0xFC08_63FC_CB14_7576,
        offset: 4608,
    },
    Key {
        shift: 59,
        magic: 0x0000_0040_8400_8881,
        offset: 4624,
    },
    Key {
        shift: 59,
        magic: 0x0058_2000_4394_0414,
        offset: 4656,
    },
    Key {
        shift: 59,
        magic: 0x0020_0000_2950_2C00,
        offset: 4688,
    },
    Key {
        shift: 59,
        magic: 0x4080_0141_0C04_1420,
        offset: 4720,
    },
    Key {
        shift: 60,
        magic: 0xFC08_7E8E_4BB2_F736,
        offset: 4752,
    },
    Key {
        shift: 59,
        magic: 0x43FF_9E4E_F4CA_2C89,
        offset: 4768,
    },
];

/// Size of the bishop move lookup table.
pub const BISHOP_TABLE_SIZE: usize = 4800;
