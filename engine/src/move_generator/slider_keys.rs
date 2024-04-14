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
        magic: 2630102251678467200,
        shift: 52,
        offset: 0,
    },
    Key {
        magic: 18084767790534656,
        shift: 53,
        offset: 4096,
    },
    Key {
        magic: 72075461238194944,
        shift: 53,
        offset: 6144,
    },
    Key {
        magic: 72067507090891008,
        shift: 53,
        offset: 8192,
    },
    Key {
        magic: 9367491657607422465,
        shift: 53,
        offset: 10240,
    },
    Key {
        magic: 2450103332857972752,
        shift: 53,
        offset: 12288,
    },
    Key {
        magic: 324299855109292544,
        shift: 53,
        offset: 14336,
    },
    Key {
        magic: 396317061414658304,
        shift: 52,
        offset: 16384,
    },
    Key {
        magic: 1171076813477150723,
        shift: 53,
        offset: 20480,
    },
    Key {
        magic: 10274111529943168,
        shift: 54,
        offset: 22528,
    },
    Key {
        magic: 1297318305099465472,
        shift: 54,
        offset: 23552,
    },
    Key {
        magic: 281513738375168,
        shift: 54,
        offset: 24576,
    },
    Key {
        magic: 36170084397368320,
        shift: 54,
        offset: 25600,
    },
    Key {
        magic: 562984582382080,
        shift: 54,
        offset: 26624,
    },
    Key {
        magic: 613615483862909507,
        shift: 54,
        offset: 27648,
    },
    Key {
        magic: 140741783381376,
        shift: 53,
        offset: 28672,
    },
    Key {
        magic: 739421708092048,
        shift: 53,
        offset: 30720,
    },
    Key {
        magic: 6989657265569554442,
        shift: 54,
        offset: 32768,
    },
    Key {
        magic: 9304437382052843520,
        shift: 54,
        offset: 33792,
    },
    Key {
        magic: 1271585331761152,
        shift: 54,
        offset: 34816,
    },
    Key {
        magic: 2252899409201168,
        shift: 54,
        offset: 35840,
    },
    Key {
        magic: 689192030298964481,
        shift: 54,
        offset: 36864,
    },
    Key {
        magic: 4684469290304669712,
        shift: 54,
        offset: 37888,
    },
    Key {
        magic: 9229852558992883843,
        shift: 53,
        offset: 38912,
    },
    Key {
        magic: 19210673604165665,
        shift: 53,
        offset: 40960,
    },
    Key {
        magic: 581563590067093637,
        shift: 54,
        offset: 43008,
    },
    Key {
        magic: 4900092335767946373,
        shift: 54,
        offset: 44032,
    },
    Key {
        magic: 11529223852899959168,
        shift: 54,
        offset: 45056,
    },
    Key {
        magic: 1446959458027520,
        shift: 54,
        offset: 46080,
    },
    Key {
        magic: 1784833527402201096,
        shift: 54,
        offset: 47104,
    },
    Key {
        magic: 6070869907061416456,
        shift: 54,
        offset: 48128,
    },
    Key {
        magic: 140756815724800,
        shift: 53,
        offset: 49152,
    },
    Key {
        magic: 4647785184735791744,
        shift: 53,
        offset: 51200,
    },
    Key {
        magic: 2310349632510631936,
        shift: 54,
        offset: 53248,
    },
    Key {
        magic: 27022456761880577,
        shift: 54,
        offset: 54272,
    },
    Key {
        magic: 72638136596832512,
        shift: 54,
        offset: 55296,
    },
    Key {
        magic: 2308235563703994368,
        shift: 54,
        offset: 56320,
    },
    Key {
        magic: 649644280809332752,
        shift: 54,
        offset: 57344,
    },
    Key {
        magic: 2305869466816227078,
        shift: 54,
        offset: 58368,
    },
    Key {
        magic: 281773778927876,
        shift: 53,
        offset: 59392,
    },
    Key {
        magic: 4972044359512391680,
        shift: 53,
        offset: 61440,
    },
    Key {
        magic: 1153009500433694724,
        shift: 54,
        offset: 63488,
    },
    Key {
        magic: 81065343316951072,
        shift: 54,
        offset: 64512,
    },
    Key {
        magic: 17596483174411,
        shift: 54,
        offset: 65536,
    },
    Key {
        magic: 5226427389038886928,
        shift: 54,
        offset: 66560,
    },
    Key {
        magic: 562984917270608,
        shift: 54,
        offset: 67584,
    },
    Key {
        magic: 3333164367878,
        shift: 54,
        offset: 68608,
    },
    Key {
        magic: 4675017890347417602,
        shift: 53,
        offset: 69632,
    },
    Key {
        magic: 5260202827150502400,
        shift: 54,
        offset: 71680,
    },
    Key {
        magic: 5260202827150502400,
        shift: 55,
        offset: 72704,
    },
    Key {
        magic: 5296232809593843200,
        shift: 55,
        offset: 73216,
    },
    Key {
        magic: 7007600874156364288,
        shift: 55,
        offset: 73728,
    },
    Key {
        magic: 18446743979218685440,
        shift: 55,
        offset: 74240,
    },
    Key {
        magic: 18446744030759085568,
        shift: 55,
        offset: 74752,
    },
    Key {
        magic: 1125444202439872,
        shift: 55,
        offset: 75264,
    },
    Key {
        magic: 5841168673586058912,
        shift: 54,
        offset: 75776,
    },
    Key {
        magic: 17005591892296975654,
        shift: 53,
        offset: 76800,
    },
    Key {
        magic: 7061642970158444206,
        shift: 54,
        offset: 78848,
    },
    Key {
        magic: 6034823423364870562,
        shift: 54,
        offset: 79872,
    },
    Key {
        magic: 1333065189051839990,
        shift: 54,
        offset: 80896,
    },
    Key {
        magic: 4692750665688806614,
        shift: 54,
        offset: 81920,
    },
    Key {
        magic: 77968723168331793,
        shift: 53,
        offset: 82944,
    },
    Key {
        magic: 1125827562356340,
        shift: 54,
        offset: 84992,
    },
    Key {
        magic: 8522499339677771678,
        shift: 53,
        offset: 86016,
    },
];

/// Size of the rook move lookup table.
pub const ROOK_TABLE_SIZE: usize = 88064;

/// Bishop magic keys.
#[allow(clippy::unreadable_literal)]
pub const BISHOP_KEYS: [Key; 64] = [
    Key {
        magic: 18441670916271046655,
        shift: 59,
        offset: 0,
    },
    Key {
        magic: 18161155296967783798,
        shift: 60,
        offset: 32,
    },
    Key {
        magic: 41099885340065920,
        shift: 59,
        offset: 48,
    },
    Key {
        magic: 2307040377915573257,
        shift: 59,
        offset: 80,
    },
    Key {
        magic: 37454038966468864,
        shift: 59,
        offset: 112,
    },
    Key {
        magic: 1165500583297552,
        shift: 59,
        offset: 144,
    },
    Key {
        magic: 18161441449164338550,
        shift: 60,
        offset: 176,
    },
    Key {
        magic: 9222806873877118975,
        shift: 59,
        offset: 192,
    },
    Key {
        magic: 18160843177395027958,
        shift: 60,
        offset: 224,
    },
    Key {
        magic: 18160900218856208374,
        shift: 60,
        offset: 240,
    },
    Key {
        magic: 2936356887043670528,
        shift: 59,
        offset: 256,
    },
    Key {
        magic: 4758061828183949312,
        shift: 59,
        offset: 288,
    },
    Key {
        magic: 19834427428928,
        shift: 59,
        offset: 320,
    },
    Key {
        magic: 82863800542298113,
        shift: 59,
        offset: 352,
    },
    Key {
        magic: 18160876197363646326,
        shift: 60,
        offset: 384,
    },
    Key {
        magic: 4325813748086734710,
        shift: 60,
        offset: 400,
    },
    Key {
        magic: 8340696151287451643,
        shift: 60,
        offset: 416,
    },
    Key {
        magic: 4728811472401641468,
        shift: 60,
        offset: 432,
    },
    Key {
        magic: 1126458254821888,
        shift: 57,
        offset: 448,
    },
    Key {
        magic: 4613938102252093504,
        shift: 57,
        offset: 576,
    },
    Key {
        magic: 562967142990080,
        shift: 57,
        offset: 704,
    },
    Key {
        magic: 565153406190594,
        shift: 57,
        offset: 832,
    },
    Key {
        magic: 8938522175157370742,
        shift: 60,
        offset: 960,
    },
    Key {
        magic: 18161331257755361142,
        shift: 60,
        offset: 976,
    },
    Key {
        magic: 9044583791460897,
        shift: 59,
        offset: 992,
    },
    Key {
        magic: 5190471139927539984,
        shift: 59,
        offset: 1024,
    },
    Key {
        magic: 4702927943155351586,
        shift: 57,
        offset: 1056,
    },
    Key {
        magic: 5981915001158500416,
        shift: 55,
        offset: 1184,
    },
    Key {
        magic: 1163621951801950720,
        shift: 55,
        offset: 1696,
    },
    Key {
        magic: 595893521903780128,
        shift: 57,
        offset: 2208,
    },
    Key {
        magic: 1770209322690971648,
        shift: 59,
        offset: 2336,
    },
    Key {
        magic: 3765080211348129825,
        shift: 59,
        offset: 2368,
    },
    Key {
        magic: 36319103660609536,
        shift: 59,
        offset: 2400,
    },
    Key {
        magic: 282608857518146,
        shift: 59,
        offset: 2432,
    },
    Key {
        magic: 2597451377148428580,
        shift: 57,
        offset: 2464,
    },
    Key {
        magic: 1801458577005871168,
        shift: 55,
        offset: 2592,
    },
    Key {
        magic: 4629726840121589792,
        shift: 55,
        offset: 3104,
    },
    Key {
        magic: 1153488870860916738,
        shift: 57,
        offset: 3616,
    },
    Key {
        magic: 4648071059365627904,
        shift: 59,
        offset: 3744,
    },
    Key {
        magic: 9224520002156650624,
        shift: 59,
        offset: 3776,
    },
    Key {
        magic: 15920182580465156255,
        shift: 60,
        offset: 3808,
    },
    Key {
        magic: 17969356424471207979,
        shift: 60,
        offset: 3824,
    },
    Key {
        magic: 360573848145380354,
        shift: 57,
        offset: 3840,
    },
    Key {
        magic: 36627071249704960,
        shift: 57,
        offset: 3968,
    },
    Key {
        magic: 10124663850176348228,
        shift: 57,
        offset: 4096,
    },
    Key {
        magic: 630591926250315840,
        shift: 57,
        offset: 4224,
    },
    Key {
        magic: 4899804643636939777,
        shift: 60,
        offset: 4352,
    },
    Key {
        magic: 5476321683761034753,
        shift: 60,
        offset: 4368,
    },
    Key {
        magic: 18163002480944018806,
        shift: 60,
        offset: 4384,
    },
    Key {
        magic: 18161881288420947318,
        shift: 60,
        offset: 4400,
    },
    Key {
        magic: 4505799758446593,
        shift: 59,
        offset: 4416,
    },
    Key {
        magic: 4407811375144,
        shift: 59,
        offset: 4448,
    },
    Key {
        magic: 1729384525192110080,
        shift: 59,
        offset: 4480,
    },
    Key {
        magic: 1155182212585488392,
        shift: 59,
        offset: 4512,
    },
    Key {
        magic: 14123209112897096841,
        shift: 60,
        offset: 4544,
    },
    Key {
        magic: 14123159053945941129,
        shift: 60,
        offset: 4560,
    },
    Key {
        magic: 18446740762247425535,
        shift: 59,
        offset: 4576,
    },
    Key {
        magic: 18160875434761549174,
        shift: 60,
        offset: 4608,
    },
    Key {
        magic: 277092534401,
        shift: 59,
        offset: 4624,
    },
    Key {
        magic: 24804983456400404,
        shift: 59,
        offset: 4656,
    },
    Key {
        magic: 9007199947860992,
        shift: 59,
        offset: 4688,
    },
    Key {
        magic: 4647716194332447776,
        shift: 59,
        offset: 4720,
    },
    Key {
        magic: 18160904646992000822,
        shift: 60,
        offset: 4752,
    },
    Key {
        magic: 4899808981553917065,
        shift: 59,
        offset: 4768,
    },
];

/// Size of the bishop move lookup table.
pub const BISHOP_TABLE_SIZE: usize = 4800;
