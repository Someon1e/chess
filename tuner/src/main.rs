use engine::board::Board;
use engine::search::eval::Eval;
use rayon::prelude::*;
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
    middle_game_piece_square_tables: &[i32; 384],
    end_game_piece_square_tables: &[i32; 384],
    phases: &[i32; 5],
) -> f64 {
    let error: f64 = data_set
        .par_iter()
        .map(|(board, result)| {
            let score = f64::from(
                Eval::evaluate(
                    middle_game_piece_square_tables,
                    end_game_piece_square_tables,
                    phases,
                    board,
                ) * if board.white_to_move { 1 } else { -1 },
            );

            let sigmoid = 1.0 / (1.0 + f64::powf(10.0, -k * score / 400.0));

            (result - sigmoid) * (result - sigmoid)
        })
        .sum();

    error / data_set.len() as f64
}

fn pretty_piece_square_tables(piece_square_tables: [i32; 384]) -> String {
    let mut output = String::new();
    output.push_str("[\n");
    for piece in 0..6 {
        for rank in 0..8 {
            output.push('\n');
            for file in 0..8 {
                output.push_str(&format!(
                    "{:>4},",
                    piece_square_tables[piece * 64 + rank * 8 + file]
                ));
            }
        }
        output.push_str("\n\n");
    }
    output.push(']');
    output
}

fn tune(
    data_set: &[(Board, f64)],
    k: f64,
    middle_game_piece_square_tables: &[i32; 384],
    end_game_piece_square_tables: &[i32; 384],
    phases: &[i32; 5],
) {
    const PSQT_ADJUSTMENT_VALUE: i32 = 1;
    const PHASE_ADJUSTMENT_VALUE: i32 = 1;

    let mut best_error = mean_square_error(
        data_set,
        k,
        middle_game_piece_square_tables,
        end_game_piece_square_tables,
        phases,
    );
    println!("Currently {best_error}");

    let log_params = |psqt_1, psqt_2, new_phases| {
        std::fs::write(
            "tuned.rs",
            format!(
                "const MIDDLE_GAME_PIECE_SQUARE_TABLES: [i32; 384] = {};
const END_GAME_PIECE_SQUARE_TABLES: [i32; 384] = {};
const PHASES: [i32; 5] = {:#?};",
                pretty_piece_square_tables(psqt_1),
                pretty_piece_square_tables(psqt_2),
                new_phases
            ),
        )
        .unwrap();
    };
    log_params(
        *middle_game_piece_square_tables,
        *end_game_piece_square_tables,
        *phases,
    );

    let mut best_psqt = [
        *middle_game_piece_square_tables,
        *end_game_piece_square_tables,
    ];
    let mut best_phases = *phases;
    let mut improved = true;

    while improved {
        improved = false;

        for table_number in 0..2 {
            for index in 0..384 {
                let mut new_psqts: [[i32; 384]; 2] = best_psqt;
                new_psqts[table_number][index] += PSQT_ADJUSTMENT_VALUE;

                let mut new_error =
                    mean_square_error(data_set, k, &new_psqts[0], &new_psqts[1], phases);

                if new_error < best_error {
                    println!("{new_error} Found better params +");
                } else {
                    new_psqts[table_number][index] -= PSQT_ADJUSTMENT_VALUE * 2;
                    new_error =
                        mean_square_error(data_set, k, &new_psqts[0], &new_psqts[1], phases);

                    if new_error < best_error {
                        println!("{new_error} Found better params -");
                    } else {
                        continue;
                    }
                }

                improved = true;
                best_error = new_error;
                best_psqt = new_psqts;
                log_params(best_psqt[0], best_psqt[1], *phases);
            }
        }
        for index in 0..5 {
            let mut new_phases = best_phases;

            let mut new_error =
                mean_square_error(data_set, k, &best_psqt[0], &best_psqt[1], &new_phases);

            if new_error < best_error {
                println!("{new_error} Found better params +");
            } else {
                new_phases[index] -= PHASE_ADJUSTMENT_VALUE * 2;
                new_error =
                    mean_square_error(data_set, k, &best_psqt[0], &best_psqt[1], &new_phases);

                if new_error < best_error {
                    println!("{new_error} Found better params -");
                } else {
                    continue;
                }
            }

            improved = true;
            best_error = new_error;
            best_phases = new_phases;
            log_params(best_psqt[0], best_psqt[1], best_phases);
        }

        println!("Finished one iteration");
    }
}

fn find_k(
    data_set: &[(Board, f64)],
    middle_game_piece_square_tables: &[i32; 384],
    end_game_piece_square_tables: &[i32; 384],
    phases: &[i32; 5],
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
                phases,
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
    let middle_game_piece_square_tables: [i32; 384] = [
        0,   0,   0,   0,   0,   0,   0,   0,
      165, 190, 167, 189, 160, 168,  89,  67,
       79,  92, 123, 130, 132, 152, 135,  89,
       61,  84,  86,  88, 109, 101, 107,  80,
       50,  76,  74,  90,  91,  84,  93,  67,
       49,  72,  71,  72,  87,  78, 107,  74,
       49,  72,  67,  55,  77,  94, 118,  66,
        0,   0,   0,   0,   0,   0,   0,   0,
     
     
      151, 230, 280, 311, 364, 267, 286, 220,
      312, 331, 366, 380, 366, 427, 337, 360,
      330, 373, 387, 402, 440, 441, 396, 364,
      332, 346, 372, 394, 376, 402, 359, 367,
      320, 333, 351, 352, 361, 357, 355, 331,
      301, 323, 338, 341, 353, 343, 345, 316,
      287, 299, 316, 329, 330, 331, 318, 314,
      240, 297, 284, 300, 304, 317, 298, 271,
     
 
      334, 323, 321, 299, 305, 311, 365, 310,
      351, 379, 373, 354, 386, 387, 375, 367,
      365, 393, 391, 417, 405, 436, 415, 399,
      359, 376, 397, 411, 405, 401, 377, 362,
      354, 367, 376, 394, 392, 377, 368, 363,
      366, 374, 373, 377, 377, 375, 374, 379,
      369, 369, 381, 358, 367, 379, 388, 370,
      344, 367, 349, 341, 346, 346, 368, 357,
     
 
      520, 517, 520, 527, 543, 536, 532, 557,
      499, 502, 523, 541, 528, 554, 533, 564,
      478, 500, 502, 505, 531, 533, 570, 542,
      462, 476, 479, 488, 493, 495, 500, 496,
      445, 445, 456, 471, 469, 455, 477, 466,
      438, 449, 455, 456, 461, 459, 492, 467,
      435, 448, 462, 458, 463, 465, 480, 444,
      454, 457, 466, 472, 475, 465, 473, 446,
     
     
      988,1011,1043,1057,1075,1075,1086,1040,
     1019,1001,1014,1013,1019,1058,1033,1083,
     1025,1023,1028,1042,1053,1092,1097,1092,
     1009,1016,1019,1019,1024,1040,1038,1043,
     1017,1012,1014,1022,1022,1021,1031,1035,
     1011,1023,1017,1017,1019,1027,1039,1031,
     1012,1019,1029,1029,1028,1035,1038,1043,
     1013,1001,1008,1024,1014,1002,1015,1004,
     
 
      -37,  15,  13, -37, -40, -18,  31,   4,
      -20, -22, -49,  12, -11,  -3, -13, -14,
      -55,  13, -33, -39, -17,  28,  26, -11,
      -54, -49, -70, -90, -76, -62, -63, -86,
      -70, -65, -89,-106,-109, -81, -87,-101,
      -35, -23, -71, -79, -79, -75, -42, -60,
       38,   0, -13, -48, -49, -35,  11,  17,
       29,  59,  32, -73,  -5, -47,  31,  33,
     
    ];

    #[rustfmt::skip]
    let end_game_piece_square_tables: [i32; 384] = [
        0,   0,   0,   0,   0,   0,   0,   0,
      246, 236, 238, 196, 200, 197, 243, 255,
      198, 204, 174, 154, 147, 134, 174, 177,
      135, 126, 108, 100,  92,  95, 111, 114,
      112, 110,  94,  92,  89,  91, 101,  96,
      106, 108,  93, 104,  97,  94,  99,  93,
      111, 112, 100, 108, 111,  99,  97,  95,
        0,   0,   0,   0,   0,   0,   0,   0,
 
      230, 262, 276, 270, 266, 260, 257, 198,
      259, 278, 278, 280, 271, 258, 270, 239,
      272, 281, 296, 295, 280, 276, 271, 257,
      279, 297, 307, 307, 309, 302, 294, 269,
      278, 290, 308, 309, 310, 301, 287, 268,
      264, 281, 288, 303, 300, 283, 275, 266,
      259, 272, 279, 279, 280, 277, 263, 264,
      253, 233, 265, 268, 267, 256, 242, 242,
     
     
      289, 297, 300, 306, 301, 296, 285, 287,
      277, 293, 297, 299, 291, 287, 296, 273,
      302, 295, 305, 294, 299, 299, 292, 293,
      297, 310, 305, 315, 311, 308, 306, 295,
      293, 309, 313, 312, 311, 309, 304, 283,
      290, 298, 308, 305, 311, 304, 291, 281,
      286, 285, 283, 298, 299, 289, 287, 270,
      273, 285, 268, 289, 285, 283, 276, 258,
     
     
      528, 532, 542, 537, 530, 528, 525, 518,
      529, 538, 541, 533, 533, 522, 521, 507,
      530, 532, 534, 531, 521, 514, 506, 504,
      533, 531, 539, 534, 521, 516, 514, 511,
      525, 532, 532, 528, 525, 524, 512, 509,
      521, 519, 520, 524, 519, 512, 493, 497,
      516, 519, 519, 521, 513, 509, 500, 513,
      508, 516, 524, 522, 515, 510, 514, 508,
     
     
      955, 955, 970, 973, 957, 956, 921, 940,
      927, 963, 990,1002,1019, 978, 969, 939,
      930, 949, 979, 986, 994, 978, 939, 929,
      941, 963, 974, 998,1005, 991, 979, 961,
      927, 965, 967, 987, 983, 974, 961, 946,
      927, 931, 957, 955, 959, 950, 931, 924,
      919, 915, 911, 921, 922, 904, 881, 868,
      908, 915, 918, 900, 915, 914, 899, 905,
     
     
      -72, -46, -28,   1,  -6,   1,  -5, -78,
      -16,  18,  27,  19,  33,  42,  41,  11,
        4,  24,  40,  47,  50,  47,  42,  16,
       -2,  28,  45,  53,  53,  50,  43,  19,
       -8,  16,  38,  50,  51,  39,  29,  12,
      -17,   4,  22,  32,  33,  25,  10,   0,
      -32,  -9,   1,  12,  15,   8,  -8, -23,
      -60, -48, -29, -11, -35, -12, -34, -61,
    ];

    let phases: [i32; 5] = [
        000, // Pawn
        100, // Knight
        100, // Bishop
        200, // Rook
        400, // Queen
    ];

    let time = Instant::now();

    let data_set = parse_data_set();

    let k = find_k(
        &data_set,
        &middle_game_piece_square_tables,
        &end_game_piece_square_tables,
        &phases,
    );

    println!("Found k: {k} in {} seconds", time.elapsed().as_secs_f64());

    tune(
        &data_set,
        k,
        &middle_game_piece_square_tables,
        &end_game_piece_square_tables,
        &phases,
    );

    println!("Tuned in {} seconds", time.elapsed().as_secs_f64());
}
