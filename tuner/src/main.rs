use engine::board::Board;
use engine::search::eval::Eval;
use std::io::BufRead;
use std::time::Instant;
use std::{fs::File, io::BufReader};

fn parse_data_set() -> Vec<(Board, f64)> {
    let file = File::open("dataset/positions.txt").expect("Failed to open file");
    let data_set = BufReader::new(file);
    let mut parsed = Vec::with_capacity(100000);

    for data in data_set.lines() {
        let Result::Ok(data) = data else {
            eprintln!("Failed to read data");
            continue;
        };

        let fen = &data[0..data.len() - 3];
        let result = &data[data.len() - 4..data.len() - 1];
        let result: f64 = match result {
            "0.0" => 0.0,
            "0.5" => 0.5,
            "1.0" => 1.0,
            _ => panic!("Unknown game result {result}"),
        };

        let board = Board::from_fen(fen);
        parsed.push((board, result));
    }

    parsed
}

fn mean_square_error(
    data_set: &[(Board, f64)],
    k: f64,
    middle_game_piece_square_tables: &[[i32; 64]; 6],
    end_game_piece_square_tables: &[[i32; 64]; 6],
) -> f64 {
    let mut error = 0.0;

    for (board, result) in data_set {
        let score = f64::from(
            Eval::evaluate(
                middle_game_piece_square_tables,
                end_game_piece_square_tables,
                board,
            ) * if board.white_to_move { 1 } else { -1 },
        );

        let sigmoid = 1.0 / (1.0 + f64::powf(10.0, -k * score / 400.0));

        error += (result - sigmoid) * (result - sigmoid);
    }

    error / data_set.len() as f64
}

fn pretty_piece_square_tables(piece_square_tables: [[i32; 64]; 6]) -> String {
    let mut output = String::new();
    output.push_str("[\n");
    for piece_square_table in piece_square_tables {
        output.push('[');
        for rank in 0..8 {
            output.push('\n');
            for file in 0..8 {
                output.push_str(&format!("{:>4},", piece_square_table[rank * 8 + file]));
            }
        }
        output.push_str("\n],\n");
    }
    output.push(']');
    output
}

fn tune(
    data_set: &[(Board, f64)],
    k: f64,
    middle_game_piece_square_tables: &[[i32; 64]; 6],
    end_game_piece_square_tables: &[[i32; 64]; 6],
) {
    const ADJUSTMENT_VALUE: i32 = 1;
    let mut best_error = mean_square_error(
        data_set,
        k,
        middle_game_piece_square_tables,
        end_game_piece_square_tables,
    );

    let log_params = |a, b| {
        std::fs::write(
            "tuned.rs",
            format!(
                "const MIDDLE_GAME_PIECE_SQUARE_TABLE: [[i32; 64]; 6] = {};
const END_GAME_PIECE_SQUARE_TABLE: [[i32; 64]; 6] = {};",
                pretty_piece_square_tables(a),
                pretty_piece_square_tables(b)
            ),
        )
        .unwrap();
    };
    log_params(
        *middle_game_piece_square_tables,
        *end_game_piece_square_tables,
    );

    let mut best_params = [
        *middle_game_piece_square_tables,
        *end_game_piece_square_tables,
    ];
    let mut improved = true;

    while improved {
        improved = false;

        for table_number in 0..2 {
            for piece in 0..6 {
                for square in 0..64 {
                    let mut new_params: [[[i32; 64]; 6]; 2] = best_params;
                    new_params[table_number][piece][square] += ADJUSTMENT_VALUE;

                    let mut new_error =
                        mean_square_error(data_set, k, &new_params[0], &new_params[1]);

                    if new_error < best_error {
                        println!("{new_error} Found better params +");
                    } else {
                        new_params[table_number][piece][square] -= ADJUSTMENT_VALUE * 2;
                        new_error = mean_square_error(data_set, k, &new_params[0], &new_params[1]);

                        if new_error < best_error {
                            println!("{new_error} Found better params -");
                        } else {
                            continue;
                        }
                    }

                    improved = true;
                    best_error = new_error;
                    best_params = new_params;
                    log_params(best_params[0], best_params[1]);
                }
            }
        }

        println!("Finished");
    }
}

fn find_k(
    data_set: &[(Board, f64)],
    middle_game_piece_square_tables: &[[i32; 64]; 6],
    end_game_piece_square_tables: &[[i32; 64]; 6],
) -> f64 {
    let mut min = -10.0;
    let mut max = 10.0;
    let mut delta = 1.0;

    let mut best = 1.0;
    let mut best_error = 100.0;

    for _ in 0..10 {
        println!("Determining K: ({min} to {max}, {delta})");

        while min < max {
            let error = mean_square_error(
                data_set,
                min,
                middle_game_piece_square_tables,
                end_game_piece_square_tables,
            );
            if error < best_error {
                best_error = error;
                best = min;
                println!("New best K: {min}, Error: {best_error}");
            }
            min += delta;
        }

        min = best - delta;
        max = best + delta;
        delta /= 10.0;
    }

    best
}

fn main() {
    #[rustfmt::skip]
    let middle_game_piece_square_tables: [[i32; 64]; 6] = [
    [
        82,  82,  82,  82,  82,  82,  82,  82,
        180, 216, 143, 177, 150, 208, 116,  71,
        76,  89, 108, 113, 147, 138, 107,  62,
        68,  95,  88, 103, 105,  94,  99,  59,
        55,  80,  77,  94,  99,  88,  92,  57,
        56,  78,  78,  72,  85,  85, 115,  70,
        47,  81,  62,  59,  67, 106, 120,  60,
        82,  82,  82,  82,  82,  82,  82,  82,
    ],
    [
        170, 248, 303, 288, 398, 240, 322, 230,
        264, 296, 409, 373, 360, 399, 344, 320,
        290, 397, 374, 402, 421, 466, 410, 381,
        328, 354, 356, 390, 374, 406, 355, 359,
        324, 341, 353, 350, 365, 356, 358, 329,
        314, 328, 349, 347, 356, 354, 362, 321,
        308, 284, 325, 334, 336, 355, 323, 318,
        232, 316, 279, 304, 320, 309, 318, 314,
    ],
    [
        336, 369, 283, 328, 340, 323, 372, 357,
        339, 381, 347, 352, 395, 424, 383, 318,
        349, 402, 408, 405, 400, 415, 402, 363,
        361, 370, 384, 415, 402, 402, 372, 363,
        359, 378, 378, 391, 399, 377, 375, 369,
        365, 380, 380, 380, 379, 392, 383, 375,
        369, 380, 381, 365, 372, 386, 398, 366,
        332, 362, 351, 344, 352, 353, 326, 344,
    ],
    [
        509, 519, 509, 528, 540, 486, 508, 520,
        504, 509, 535, 539, 557, 544, 503, 521,
        472, 496, 503, 513, 494, 522, 538, 493,
        453, 466, 484, 503, 501, 512, 469, 457,
        441, 451, 465, 476, 486, 470, 483, 454,
        432, 452, 461, 460, 480, 477, 472, 444,
        433, 461, 457, 468, 476, 488, 471, 406,
        458, 464, 478, 494, 493, 484, 440, 451,
    ],
    [
        997,1025,1054,1037,1084,1069,1068,1070,
        1001, 986,1020,1026,1009,1082,1053,1079,
        1012,1008,1032,1033,1054,1081,1072,1082,
        998, 998,1009,1009,1024,1042,1023,1026,
        1016, 999,1016,1015,1023,1021,1028,1022,
        1011,1027,1014,1023,1020,1027,1039,1030,
        990,1017,1036,1027,1033,1040,1022,1026,
        1024,1007,1016,1035,1010,1000, 994, 975,
    ],
    [
        -65,  23,  16, -15, -56, -34,   2,  13,
        29,  -1, -20,  -7,  -8,  -4, -38, -29,
        -9,  24,   2, -16, -20,   6,  22, -22,
        -17, -20, -12, -27, -30, -25, -14, -36,
        -49,  -1, -27, -39, -46, -44, -33, -51,
        -14, -14, -22, -46, -44, -30, -15, -27,
        1,   7,  -8, -64, -43, -16,   9,   8,
        -15,  36,  12, -54,   8, -28,  24,  14,
        ],
    ];

    #[rustfmt::skip]
    let end_game_piece_square_tables: [[i32; 64]; 6] = [
    [
        94,  94,  94,  94,  94,  94,  94,  94,
        272, 267, 252, 228, 241, 226, 259, 281,
        188, 194, 179, 161, 150, 147, 176, 178,
        126, 118, 107,  99,  92,  98, 111, 111,
        107, 103,  91,  87,  87,  86,  97,  93,
        98, 101,  88,  95,  94,  89,  93,  86,
        107, 102, 102, 104, 107,  94,  96,  87,
        94,  94,  94,  94,  94,  94,  94,  94,
    ],
    [
        223, 243, 268, 253, 250, 254, 218, 182,
        256, 273, 256, 279, 272, 256, 257, 229,
        257, 261, 291, 290, 280, 272, 262, 240,
        264, 284, 303, 303, 303, 292, 289, 263,
        263, 275, 297, 306, 297, 298, 285, 263,
        258, 278, 280, 296, 291, 278, 261, 259,
        239, 261, 271, 276, 279, 261, 258, 237,
        252, 230, 258, 266, 259, 263, 231, 217,
    ],
    [
        283, 276, 286, 289, 290, 288, 280, 273,
        289, 293, 304, 285, 294, 284, 293, 283,
        299, 289, 297, 296, 295, 303, 297, 301,
        294, 306, 309, 306, 311, 307, 300, 299,
        291, 300, 310, 316, 304, 307, 294, 288,
        285, 294, 305, 307, 310, 300, 290, 282,
        283, 279, 290, 296, 301, 288, 282, 270,
        274, 288, 274, 292, 288, 281, 292, 280,
    ],
    [
        525, 522, 530, 527, 524, 524, 520, 517,
        523, 525, 525, 523, 509, 515, 520, 515,
        519, 519, 519, 517, 516, 509, 507, 509,
        516, 515, 525, 513, 514, 513, 511, 514,
        515, 517, 520, 516, 507, 506, 504, 501,
        508, 512, 507, 511, 505, 500, 504, 496,
        506, 506, 512, 514, 503, 503, 501, 509,
        503, 514, 515, 511, 507, 499, 516, 492,
    ],
    [
        927, 958, 958, 963, 963, 955, 946, 956,
        919, 956, 968, 977, 994, 961, 966, 936,
        916, 942, 945, 985, 983, 971, 955, 945,
        939, 958, 960, 981, 993, 976, 993, 972,
        918, 964, 955, 983, 967, 970, 975, 959,
        920, 909, 951, 942, 945, 953, 946, 941,
        914, 913, 906, 920, 920, 913, 900, 904,
        903, 908, 914, 893, 931, 904, 916, 895,
    ],
    [
        -74, -35, -18, -18, -11,  15,   4, -17,
        -12,  17,  14,  17,  17,  38,  23,  11,
        10,  17,  23,  15,  20,  45,  44,  13,
        -8,  22,  24,  27,  26,  33,  26,   3,
        -18,  -4,  21,  24,  27,  23,   9, -11,
        -19,  -3,  11,  21,  23,  16,   7,  -9,
        -27, -11,   4,  13,  14,   4,  -5, -17,
        -53, -34, -21, -11, -28, -14, -24, -43,
        ],
    ];

    let time = Instant::now();

    let k = find_k(
        &parse_data_set(),
        &middle_game_piece_square_tables,
        &end_game_piece_square_tables,
    );

    println!("{k}");
    println!("Found k in {} seconds", time.elapsed().as_secs_f64());

    tune(
        &parse_data_set(),
        0.2,
        &middle_game_piece_square_tables,
        &end_game_piece_square_tables,
    );

    println!("Tuned in {} seconds", time.elapsed().as_secs_f64());
}
