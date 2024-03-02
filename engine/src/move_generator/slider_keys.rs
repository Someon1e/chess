#[derive(Debug, Clone, Copy)]
pub struct Key {
    pub magic: u64,
    pub shift: u8,
    pub offset: u32,
}

pub const ROOK_KEYS: [Key; 64] = [
    Key {
        magic: 72075735996571841,
        shift: 52,
        offset: 0,
    },
    Key {
        magic: 18014674998071298,
        shift: 53,
        offset: 4096,
    },
    Key {
        magic: 9295439526767886400,
        shift: 53,
        offset: 6144,
    },
    Key {
        magic: 72093088789561600,
        shift: 53,
        offset: 8192,
    },
    Key {
        magic: 11565252645623051264,
        shift: 53,
        offset: 10240,
    },
    Key {
        magic: 144121785213813032,
        shift: 53,
        offset: 12288,
    },
    Key {
        magic: 288239242105129090,
        shift: 53,
        offset: 14336,
    },
    Key {
        magic: 108090927624487680,
        shift: 52,
        offset: 16384,
    },
    Key {
        magic: 140772921843840,
        shift: 53,
        offset: 20480,
    },
    Key {
        magic: 85357288029691904,
        shift: 54,
        offset: 22528,
    },
    Key {
        magic: 4903012692341563394,
        shift: 54,
        offset: 23552,
    },
    Key {
        magic: 9148039844331648,
        shift: 54,
        offset: 24576,
    },
    Key {
        magic: 1153062791918135296,
        shift: 54,
        offset: 25600,
    },
    Key {
        magic: 9225764591345074688,
        shift: 54,
        offset: 26624,
    },
    Key {
        magic: 2306405963673383424,
        shift: 54,
        offset: 27648,
    },
    Key {
        magic: 1153062245870077056,
        shift: 53,
        offset: 28672,
    },
    Key {
        magic: 2323857957484265512,
        shift: 53,
        offset: 30720,
    },
    Key {
        magic: 4611756662317916160,
        shift: 54,
        offset: 32768,
    },
    Key {
        magic: 576480544083165313,
        shift: 54,
        offset: 33792,
    },
    Key {
        magic: 4504149517406338,
        shift: 54,
        offset: 34816,
    },
    Key {
        magic: 1153062791985234944,
        shift: 54,
        offset: 35840,
    },
    Key {
        magic: 4620702018139340848,
        shift: 54,
        offset: 36864,
    },
    Key {
        magic: 1152925902922459137,
        shift: 54,
        offset: 37888,
    },
    Key {
        magic: 4917968176496542721,
        shift: 53,
        offset: 38912,
    },
    Key {
        magic: 29343768471699456,
        shift: 53,
        offset: 40960,
    },
    Key {
        magic: 4900162686261661696,
        shift: 54,
        offset: 43008,
    },
    Key {
        magic: 52778707714176,
        shift: 54,
        offset: 44032,
    },
    Key {
        magic: 17596481601826,
        shift: 54,
        offset: 45056,
    },
    Key {
        magic: 144396736067534852,
        shift: 54,
        offset: 46080,
    },
    Key {
        magic: 9819047856512827520,
        shift: 54,
        offset: 47104,
    },
    Key {
        magic: 37172306325799424,
        shift: 54,
        offset: 48128,
    },
    Key {
        magic: 10377420824348238849,
        shift: 53,
        offset: 49152,
    },
    Key {
        magic: 864761497811681408,
        shift: 53,
        offset: 51200,
    },
    Key {
        magic: 1152991942108258304,
        shift: 54,
        offset: 53248,
    },
    Key {
        magic: 6917674300640079872,
        shift: 54,
        offset: 54272,
    },
    Key {
        magic: 17594409027586,
        shift: 54,
        offset: 55296,
    },
    Key {
        magic: 5651491939424257,
        shift: 54,
        offset: 56320,
    },
    Key {
        magic: 1747537410096382464,
        shift: 54,
        offset: 57344,
    },
    Key {
        magic: 45194330847053826,
        shift: 54,
        offset: 58368,
    },
    Key {
        magic: 9234653032631767141,
        shift: 53,
        offset: 59392,
    },
    Key {
        magic: 1548525762543632,
        shift: 53,
        offset: 61440,
    },
    Key {
        magic: 148706749170335746,
        shift: 54,
        offset: 63488,
    },
    Key {
        magic: 1154611179713331217,
        shift: 54,
        offset: 64512,
    },
    Key {
        magic: 1162245440524124184,
        shift: 54,
        offset: 65536,
    },
    Key {
        magic: 18577933118603268,
        shift: 54,
        offset: 66560,
    },
    Key {
        magic: 4901042303084298316,
        shift: 54,
        offset: 67584,
    },
    Key {
        magic: 576498135715577984,
        shift: 54,
        offset: 68608,
    },
    Key {
        magic: 9259403583793791011,
        shift: 53,
        offset: 69632,
    },
    Key {
        magic: 162129726171859072,
        shift: 53,
        offset: 71680,
    },
    Key {
        magic: 35253129314880,
        shift: 54,
        offset: 73728,
    },
    Key {
        magic: 9817864781464699264,
        shift: 54,
        offset: 74752,
    },
    Key {
        magic: 177022581080320,
        shift: 54,
        offset: 75776,
    },
    Key {
        magic: 1491008768508160,
        shift: 54,
        offset: 76800,
    },
    Key {
        magic: 141845590179968,
        shift: 54,
        offset: 77824,
    },
    Key {
        magic: 577023723799314944,
        shift: 54,
        offset: 78848,
    },
    Key {
        magic: 72198335821529472,
        shift: 53,
        offset: 79872,
    },
    Key {
        magic: 9234985082963918882,
        shift: 52,
        offset: 81920,
    },
    Key {
        magic: 90213829822398469,
        shift: 53,
        offset: 86016,
    },
    Key {
        magic: 1143638130296898,
        shift: 53,
        offset: 88064,
    },
    Key {
        magic: 9044067522454785,
        shift: 53,
        offset: 90112,
    },
    Key {
        magic: 151433642967567426,
        shift: 53,
        offset: 92160,
    },
    Key {
        magic: 9800395756560454018,
        shift: 53,
        offset: 94208,
    },
    Key {
        magic: 184731106853380,
        shift: 53,
        offset: 96256,
    },
    Key {
        magic: 9799837188295237762,
        shift: 52,
        offset: 98304,
    },
];
pub const ROOK_TABLE_SIZE: usize = 102400;

pub const BISHOP_KEYS: [Key; 64] = [
    Key {
        magic: 10212833149092354,
        shift: 58,
        offset: 0,
    },
    Key {
        magic: 153124603818975264,
        shift: 59,
        offset: 64,
    },
    Key {
        magic: 4625201217639809256,
        shift: 59,
        offset: 96,
    },
    Key {
        magic: 2342505678981039120,
        shift: 59,
        offset: 128,
    },
    Key {
        magic: 6351205909989949952,
        shift: 59,
        offset: 160,
    },
    Key {
        magic: 9233648210505936920,
        shift: 59,
        offset: 192,
    },
    Key {
        magic: 162698086177316864,
        shift: 59,
        offset: 224,
    },
    Key {
        magic: 576533354501964802,
        shift: 58,
        offset: 256,
    },
    Key {
        magic: 4617051652619501632,
        shift: 59,
        offset: 320,
    },
    Key {
        magic: 4618725143525720320,
        shift: 59,
        offset: 352,
    },
    Key {
        magic: 1153502326997188612,
        shift: 59,
        offset: 384,
    },
    Key {
        magic: 4415368986624,
        shift: 59,
        offset: 416,
    },
    Key {
        magic: 9008372855636033,
        shift: 59,
        offset: 448,
    },
    Key {
        magic: 2305883708861054976,
        shift: 59,
        offset: 480,
    },
    Key {
        magic: 189156684189680138,
        shift: 59,
        offset: 512,
    },
    Key {
        magic: 292479564122112,
        shift: 59,
        offset: 544,
    },
    Key {
        magic: 2314850483651084896,
        shift: 59,
        offset: 576,
    },
    Key {
        magic: 2252487153493057,
        shift: 59,
        offset: 608,
    },
    Key {
        magic: 2326395114953515520,
        shift: 57,
        offset: 640,
    },
    Key {
        magic: 164381438509195264,
        shift: 57,
        offset: 768,
    },
    Key {
        magic: 2401488024371200,
        shift: 57,
        offset: 896,
    },
    Key {
        magic: 153194963709075457,
        shift: 57,
        offset: 1024,
    },
    Key {
        magic: 76649713126868996,
        shift: 59,
        offset: 1152,
    },
    Key {
        magic: 9223426260825801729,
        shift: 59,
        offset: 1184,
    },
    Key {
        magic: 4512405401387346,
        shift: 59,
        offset: 1216,
    },
    Key {
        magic: 9234798196589396737,
        shift: 59,
        offset: 1248,
    },
    Key {
        magic: 577588859890641920,
        shift: 57,
        offset: 1280,
    },
    Key {
        magic: 1153625742475657218,
        shift: 55,
        offset: 1408,
    },
    Key {
        magic: 72339413702557696,
        shift: 55,
        offset: 1920,
    },
    Key {
        magic: 211656140394496,
        shift: 57,
        offset: 2432,
    },
    Key {
        magic: 99642485420132356,
        shift: 59,
        offset: 2560,
    },
    Key {
        magic: 9871961061211637248,
        shift: 59,
        offset: 2592,
    },
    Key {
        magic: 4756399393174884352,
        shift: 59,
        offset: 2624,
    },
    Key {
        magic: 1182199480620958212,
        shift: 59,
        offset: 2656,
    },
    Key {
        magic: 19275881382080,
        shift: 57,
        offset: 2688,
    },
    Key {
        magic: 6757609204351104,
        shift: 55,
        offset: 2816,
    },
    Key {
        magic: 90081892715528256,
        shift: 55,
        offset: 3328,
    },
    Key {
        magic: 4800846017123591169,
        shift: 57,
        offset: 3840,
    },
    Key {
        magic: 310794627866330144,
        shift: 59,
        offset: 3968,
    },
    Key {
        magic: 4627449237745972736,
        shift: 59,
        offset: 4000,
    },
    Key {
        magic: 9233650409527068736,
        shift: 59,
        offset: 4032,
    },
    Key {
        magic: 577868711373840388,
        shift: 59,
        offset: 4064,
    },
    Key {
        magic: 108368691346804736,
        shift: 57,
        offset: 4096,
    },
    Key {
        magic: 1175439786219935744,
        shift: 57,
        offset: 4224,
    },
    Key {
        magic: 1297041127538557952,
        shift: 57,
        offset: 4352,
    },
    Key {
        magic: 9241409542307242241,
        shift: 57,
        offset: 4480,
    },
    Key {
        magic: 2841172415488256,
        shift: 59,
        offset: 4608,
    },
    Key {
        magic: 2884556218628575300,
        shift: 59,
        offset: 4640,
    },
    Key {
        magic: 290284224643072,
        shift: 59,
        offset: 4672,
    },
    Key {
        magic: 144609178102530080,
        shift: 59,
        offset: 4704,
    },
    Key {
        magic: 2310346754887778305,
        shift: 59,
        offset: 4736,
    },
    Key {
        magic: 6379348890459176964,
        shift: 59,
        offset: 4768,
    },
    Key {
        magic: 13511967147163648,
        shift: 59,
        offset: 4800,
    },
    Key {
        magic: 8878907064609,
        shift: 59,
        offset: 4832,
    },
    Key {
        magic: 2891319825602199552,
        shift: 59,
        offset: 4864,
    },
    Key {
        magic: 5638367098208288,
        shift: 59,
        offset: 4896,
    },
    Key {
        magic: 1411775144919552,
        shift: 58,
        offset: 4928,
    },
    Key {
        magic: 39424656062939648,
        shift: 59,
        offset: 4992,
    },
    Key {
        magic: 2323861810133469444,
        shift: 59,
        offset: 5024,
    },
    Key {
        magic: 9265349120,
        shift: 59,
        offset: 5056,
    },
    Key {
        magic: 1686051619328,
        shift: 59,
        offset: 5088,
    },
    Key {
        magic: 9007508762984962,
        shift: 59,
        offset: 5120,
    },
    Key {
        magic: 11602416821675819072,
        shift: 59,
        offset: 5152,
    },
    Key {
        magic: 6923316859299463680,
        shift: 58,
        offset: 5184,
    },
];
pub const BISHOP_TABLE_SIZE: usize = 5248;
