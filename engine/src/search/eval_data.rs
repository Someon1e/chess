pub type EvalNumber = i32;

#[rustfmt::skip]
pub const MIDDLE_GAME_PIECE_SQUARE_TABLES: [EvalNumber; 384] = [
   0,   0,   0,   0,   0,   0,   0,   0,
 158, 180, 170, 195, 180, 163,  96,  76,
  79,  93, 124, 131, 134, 154, 135,  91,
  62,  86,  87,  89, 110, 101, 105,  81,
  51,  77,  75,  90,  90,  82,  91,  68,
  49,  74,  71,  72,  87,  74, 105,  74,
  48,  74,  68,  55,  76,  92, 115,  66,
   0,   0,   0,   0,   0,   0,   0,   0,


 152, 227, 275, 320, 343, 276, 254, 210,
 312, 330, 363, 378, 364, 429, 335, 359,
 327, 369, 385, 402, 441, 442, 395, 361,
 328, 342, 368, 391, 373, 400, 357, 365,
 316, 330, 348, 349, 359, 354, 352, 327,
 297, 320, 334, 338, 350, 339, 342, 314,
 283, 296, 314, 325, 326, 329, 315, 311,
 240, 292, 280, 296, 301, 314, 294, 266,


 329, 319, 328, 298, 301, 315, 353, 301,
 349, 376, 374, 354, 384, 387, 376, 368,
 365, 389, 390, 417, 404, 432, 414, 399,
 355, 373, 394, 408, 404, 400, 375, 360,
 352, 365, 374, 393, 390, 375, 366, 361,
 364, 371, 370, 375, 375, 371, 372, 377,
 365, 367, 379, 355, 365, 377, 385, 368,
 343, 363, 346, 340, 343, 343, 368, 354,


 520, 514, 520, 529, 541, 543, 529, 558,
 500, 499, 521, 540, 526, 555, 540, 561,
 478, 501, 503, 507, 534, 533, 567, 546,
 461, 477, 478, 485, 493, 493, 503, 503,
 443, 445, 456, 468, 468, 453, 477, 468,
 436, 447, 455, 455, 460, 457, 492, 469,
 433, 447, 461, 458, 461, 463, 478, 450,
 451, 455, 465, 472, 473, 462, 480, 454,


 992,1018,1046,1067,1069,1071,1075,1026,
1026,1002,1018,1010,1020,1055,1031,1078,
1028,1031,1030,1043,1055,1087,1093,1088,
1012,1019,1022,1020,1028,1042,1039,1045,
1019,1015,1015,1022,1022,1022,1032,1037,
1014,1023,1018,1017,1021,1027,1040,1032,
1015,1020,1030,1029,1028,1035,1037,1046,
1014,1001,1008,1023,1015,1003,1018,1014,


  -7,  -1,  -6, -51, -32,   9,  34,  25,
 -38, -24, -44,   5,  -9,  -6,  14,   0,
 -65,   8, -44, -52, -30,  20,  24, -23,
 -70, -73, -82,-123,-120, -84, -80, -91,
 -75, -79,-106,-131,-137,-101, -98,-121,
 -41, -22, -81, -90, -88, -84, -39, -55,
  42,   2, -11, -48, -49, -29,  21,  29,
  35,  59,  31, -69,  -3, -38,  44,  45,
];

#[rustfmt::skip]
pub const END_GAME_PIECE_SQUARE_TABLES: [EvalNumber; 384] = [
   0,   0,   0,   0,   0,   0,   0,   0,
 252, 244, 241, 197, 194, 202, 242, 254,
 201, 207, 177, 156, 148, 135, 178, 176,
 136, 126, 109, 101,  94,  97, 115, 115,
 113, 111,  95,  94,  92,  94, 104,  97,
 108, 108,  94, 105,  99,  98, 103,  94,
 114, 112, 101, 109, 114, 102, 102,  96,
   0,   0,   0,   0,   0,   0,   0,   0,


 228, 267, 283, 271, 277, 261, 268, 206,
 263, 283, 285, 286, 278, 265, 277, 246,
 278, 290, 305, 302, 287, 284, 278, 265,
 285, 305, 315, 315, 316, 310, 299, 277,
 287, 298, 316, 316, 317, 309, 296, 276,
 270, 287, 296, 309, 308, 291, 284, 271,
 263, 277, 285, 287, 288, 283, 269, 273,
 255, 245, 272, 274, 273, 263, 251, 250,


 298, 304, 303, 312, 308, 300, 291, 296,
 284, 300, 301, 306, 297, 294, 302, 275,
 308, 303, 312, 300, 304, 307, 297, 298,
 305, 318, 313, 323, 317, 313, 313, 302,
 298, 315, 319, 318, 317, 315, 311, 288,
 297, 306, 315, 312, 318, 312, 298, 287,
 293, 291, 289, 306, 304, 296, 294, 275,
 277, 293, 274, 294, 292, 290, 280, 266,


 536, 542, 551, 546, 540, 534, 536, 528,
 536, 548, 550, 543, 543, 530, 528, 518,
 538, 541, 541, 538, 528, 523, 517, 513,
 541, 538, 547, 544, 529, 525, 521, 517,
 533, 537, 539, 538, 533, 531, 519, 515,
 528, 526, 527, 531, 526, 520, 501, 502,
 523, 526, 527, 528, 522, 518, 511, 514,
 517, 525, 532, 529, 525, 521, 514, 508,


 961, 957, 976, 972, 969, 964, 928, 960,
 925, 973, 995,1017,1029, 991, 975, 945,
 936, 947, 987, 997,1003, 992, 952, 944,
 945, 966, 982,1009,1014, 998, 986, 965,
 934, 967, 977, 999, 994, 983, 968, 950,
 929, 939, 964, 962, 963, 957, 933, 928,
 916, 921, 914, 928, 929, 909, 889, 865,
 911, 922, 926, 913, 917, 917, 903, 901,


 -74, -35, -18,   9,  -1,   3,   1, -75,
  -4,  26,  33,  26,  39,  50,  44,  15,
  13,  33,  48,  56,  58,  55,  51,  25,
   9,  39,  54,  65,  66,  61,  52,  26,
  -1,  26,  48,  61,  61,  50,  37,  22,
  -9,  11,  31,  41,  42,  34,  16,   5,
 -26,  -2,   9,  19,  22,  13,  -4, -21,
 -55, -37, -18,  -6, -27, -10, -33, -59,
];

pub const PHASES: [EvalNumber; 5] = [0, 81, 114, 174, 430];
