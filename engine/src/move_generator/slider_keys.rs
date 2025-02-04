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
        magic: 0x00280077ffebfffe,
        offset: 41305,
    },
    Key {
        magic: 0x2004010201097fff,
        offset: 14326,
    },
    Key {
        magic: 0x0010020010053fff,
        offset: 24477,
    },
    Key {
        magic: 0x0030002ff71ffffa,
        offset: 8223,
    },
    Key {
        magic: 0x7fd00441ffffd003,
        offset: 49795,
    },
    Key {
        magic: 0x004001d9e03ffff7,
        offset: 60546,
    },
    Key {
        magic: 0x004000888847ffff,
        offset: 28543,
    },
    Key {
        magic: 0x006800fbff75fffd,
        offset: 79282,
    },
    Key {
        magic: 0x000028010113ffff,
        offset: 6457,
    },
    Key {
        magic: 0x0020040201fcffff,
        offset: 4125,
    },
    Key {
        magic: 0x007fe80042ffffe8,
        offset: 81021,
    },
    Key {
        magic: 0x00001800217fffe8,
        offset: 42341,
    },
    Key {
        magic: 0x00001800073fffe8,
        offset: 14139,
    },
    Key {
        magic: 0x007fe8009effffe8,
        offset: 19465,
    },
    Key {
        magic: 0x00001800602fffe8,
        offset: 9514,
    },
    Key {
        magic: 0x000030002fffffa0,
        offset: 71090,
    },
    Key {
        magic: 0x00300018010bffff,
        offset: 75419,
    },
    Key {
        magic: 0x0003000c0085fffb,
        offset: 33476,
    },
    Key {
        magic: 0x0004000802010008,
        offset: 27117,
    },
    Key {
        magic: 0x0002002004002002,
        offset: 85964,
    },
    Key {
        magic: 0x0002002020010002,
        offset: 54915,
    },
    Key {
        magic: 0x0001002020008001,
        offset: 36544,
    },
    Key {
        magic: 0x0000004040008001,
        offset: 71854,
    },
    Key {
        magic: 0x0000802000200040,
        offset: 37996,
    },
    Key {
        magic: 0x0040200010080010,
        offset: 30398,
    },
    Key {
        magic: 0x0000080010040010,
        offset: 55939,
    },
    Key {
        magic: 0x0004010008020008,
        offset: 53891,
    },
    Key {
        magic: 0x0000040020200200,
        offset: 56963,
    },
    Key {
        magic: 0x0000010020020020,
        offset: 77451,
    },
    Key {
        magic: 0x0000010020200080,
        offset: 12319,
    },
    Key {
        magic: 0x0000008020200040,
        offset: 88500,
    },
    Key {
        magic: 0x0000200020004081,
        offset: 51405,
    },
    Key {
        magic: 0x00fffd1800300030,
        offset: 72878,
    },
    Key {
        magic: 0x007fff7fbfd40020,
        offset: 676,
    },
    Key {
        magic: 0x003fffbd00180018,
        offset: 83122,
    },
    Key {
        magic: 0x001fffde80180018,
        offset: 22206,
    },
    Key {
        magic: 0x000fffe0bfe80018,
        offset: 75186,
    },
    Key {
        magic: 0x0001000080202001,
        offset: 681,
    },
    Key {
        magic: 0x0003fffbff980180,
        offset: 36453,
    },
    Key {
        magic: 0x0001fffdff9000e0,
        offset: 20369,
    },
    Key {
        magic: 0x00fffeebfeffd800,
        offset: 1981,
    },
    Key {
        magic: 0x007ffff7ffc01400,
        offset: 13343,
    },
    Key {
        magic: 0x0000408104200204,
        offset: 10650,
    },
    Key {
        magic: 0x001ffff01fc03000,
        offset: 57987,
    },
    Key {
        magic: 0x000fffe7f8bfe800,
        offset: 26302,
    },
    Key {
        magic: 0x0000008001002020,
        offset: 58357,
    },
    Key {
        magic: 0x0003fff85fffa804,
        offset: 40546,
    },
    Key {
        magic: 0x0001fffd75ffa802,
        offset: 0,
    },
    Key {
        magic: 0x00ffffec00280028,
        offset: 14967,
    },
    Key {
        magic: 0x007fff75ff7fbfd8,
        offset: 80361,
    },
    Key {
        magic: 0x003fff863fbf7fd8,
        offset: 40905,
    },
    Key {
        magic: 0x001fffbfdfd7ffd8,
        offset: 58347,
    },
    Key {
        magic: 0x000ffff810280028,
        offset: 20381,
    },
    Key {
        magic: 0x0007ffd7f7feffd8,
        offset: 81868,
    },
    Key {
        magic: 0x0003fffc0c480048,
        offset: 59381,
    },
    Key {
        magic: 0x0001ffffafd7ffd8,
        offset: 84404,
    },
    Key {
        magic: 0x00ffffe4ffdfa3ba,
        offset: 45811,
    },
    Key {
        magic: 0x007fffef7ff3d3da,
        offset: 62898,
    },
    Key {
        magic: 0x003fffbfdfeff7fa,
        offset: 45796,
    },
    Key {
        magic: 0x001fffeff7fbfc22,
        offset: 66994,
    },
    Key {
        magic: 0x0000020408001001,
        offset: 67204,
    },
    Key {
        magic: 0x0007fffeffff77fd,
        offset: 32448,
    },
    Key {
        magic: 0x0003ffffbf7dfeec,
        offset: 62946,
    },
    Key {
        magic: 0x0001ffff9dffa333,
        offset: 17005,
    },
];

/// Bishop magic keys.
#[allow(clippy::unreadable_literal)]
pub const BISHOP_KEYS: [Key; 64] = [
    Key {
        magic: 0x0000404040404040,
        offset: 33104,
    },
    Key {
        magic: 0x0000a060401007fc,
        offset: 4094,
    },
    Key {
        magic: 0x0000401020200000,
        offset: 24764,
    },
    Key {
        magic: 0x0000806004000000,
        offset: 13882,
    },
    Key {
        magic: 0x0000440200000000,
        offset: 23090,
    },
    Key {
        magic: 0x0000080100800000,
        offset: 32640,
    },
    Key {
        magic: 0x0000104104004000,
        offset: 11558,
    },
    Key {
        magic: 0x0000020020820080,
        offset: 32912,
    },
    Key {
        magic: 0x0000040100202004,
        offset: 13674,
    },
    Key {
        magic: 0x0000020080200802,
        offset: 6109,
    },
    Key {
        magic: 0x0000010040080200,
        offset: 26494,
    },
    Key {
        magic: 0x0000008060040000,
        offset: 17919,
    },
    Key {
        magic: 0x0000004402000000,
        offset: 25757,
    },
    Key {
        magic: 0x00000021c100b200,
        offset: 17338,
    },
    Key {
        magic: 0x0000000400410080,
        offset: 16983,
    },
    Key {
        magic: 0x000003f7f05fffc0,
        offset: 16659,
    },
    Key {
        magic: 0x0004228040808010,
        offset: 13610,
    },
    Key {
        magic: 0x0000200040404040,
        offset: 2224,
    },
    Key {
        magic: 0x0000400080808080,
        offset: 60405,
    },
    Key {
        magic: 0x0000200200801000,
        offset: 7983,
    },
    Key {
        magic: 0x0000240080840000,
        offset: 17,
    },
    Key {
        magic: 0x000018000c03fff8,
        offset: 34321,
    },
    Key {
        magic: 0x00000a5840208020,
        offset: 33216,
    },
    Key {
        magic: 0x0000058408404010,
        offset: 17127,
    },
    Key {
        magic: 0x0002022000408020,
        offset: 6397,
    },
    Key {
        magic: 0x0000402000408080,
        offset: 22169,
    },
    Key {
        magic: 0x0000804000810100,
        offset: 42727,
    },
    Key {
        magic: 0x000100403c0403ff,
        offset: 155,
    },
    Key {
        magic: 0x00078402a8802000,
        offset: 8601,
    },
    Key {
        magic: 0x0000101000804400,
        offset: 21101,
    },
    Key {
        magic: 0x0000080800104100,
        offset: 29885,
    },
    Key {
        magic: 0x0000400480101008,
        offset: 29340,
    },
    Key {
        magic: 0x0001010102004040,
        offset: 19785,
    },
    Key {
        magic: 0x0000808090402020,
        offset: 12258,
    },
    Key {
        magic: 0x0007fefe08810010,
        offset: 50451,
    },
    Key {
        magic: 0x0003ff0f833fc080,
        offset: 1712,
    },
    Key {
        magic: 0x007fe08019003042,
        offset: 78475,
    },
    Key {
        magic: 0x0000202040008040,
        offset: 7855,
    },
    Key {
        magic: 0x0001004008381008,
        offset: 13642,
    },
    Key {
        magic: 0x0000802003700808,
        offset: 8156,
    },
    Key {
        magic: 0x0000208200400080,
        offset: 4348,
    },
    Key {
        magic: 0x0000104100200040,
        offset: 28794,
    },
    Key {
        magic: 0x0003ffdf7f833fc0,
        offset: 22578,
    },
    Key {
        magic: 0x0000008840450020,
        offset: 50315,
    },
    Key {
        magic: 0x0000020040100100,
        offset: 85452,
    },
    Key {
        magic: 0x007fffdd80140028,
        offset: 32816,
    },
    Key {
        magic: 0x0000202020200040,
        offset: 13930,
    },
    Key {
        magic: 0x0001004010039004,
        offset: 17967,
    },
    Key {
        magic: 0x0000040041008000,
        offset: 33200,
    },
    Key {
        magic: 0x0003ffefe0c02200,
        offset: 32456,
    },
    Key {
        magic: 0x0000001010806000,
        offset: 7762,
    },
    Key {
        magic: 0x0000000008403000,
        offset: 7794,
    },
    Key {
        magic: 0x0000000100202000,
        offset: 22761,
    },
    Key {
        magic: 0x0000040100200800,
        offset: 14918,
    },
    Key {
        magic: 0x0000404040404000,
        offset: 11620,
    },
    Key {
        magic: 0x00006020601803f4,
        offset: 15925,
    },
    Key {
        magic: 0x0003ffdfdfc28048,
        offset: 32528,
    },
    Key {
        magic: 0x0000000820820020,
        offset: 12196,
    },
    Key {
        magic: 0x0000000010108060,
        offset: 32720,
    },
    Key {
        magic: 0x0000000000084030,
        offset: 26781,
    },
    Key {
        magic: 0x0000000001002020,
        offset: 19817,
    },
    Key {
        magic: 0x0000000040408020,
        offset: 24732,
    },
    Key {
        magic: 0x0000004040404040,
        offset: 25468,
    },
    Key {
        magic: 0x0000404040404040,
        offset: 10186,
    },
];

/// Size of the move lookup table.
pub const SLIDERS_TABLE_SIZE: usize = 89524;

// Magic numbers taken from http://www.talkchess.com/forum/viewtopic.php?t=60065&start=14
