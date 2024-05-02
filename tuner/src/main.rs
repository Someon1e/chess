#![deny(clippy::all)]
#![warn(clippy::nursery)]

use engine::board::Board;
use engine::search::eval::Eval;
use rayon::prelude::*;
use std::io::BufRead;
use std::time::Instant;
use std::{fs::File, io::BufReader};

fn parse_data_set() -> Vec<(Board, f64)> {
    let file = File::open("dataset/positions.txt").expect("Failed to open file");
    let data_set = BufReader::new(file);
    let mut parsed = Vec::with_capacity(1_000_000);

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
    parsed.shrink_to_fit();

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
                    mean_square_error(data_set, k, &new_psqts[0], &new_psqts[1], &best_phases);

                if new_error < best_error {
                    println!("{new_error} Found better params +");
                } else {
                    new_psqts[table_number][index] -= PSQT_ADJUSTMENT_VALUE * 2;
                    new_error =
                        mean_square_error(data_set, k, &new_psqts[0], &new_psqts[1], &best_phases);

                    if new_error < best_error {
                        println!("{new_error} Found better params -");
                    } else {
                        continue;
                    }
                }

                improved = true;
                best_error = new_error;
                best_psqt = new_psqts;
                log_params(best_psqt[0], best_psqt[1], best_phases);
            }
        }
        for index in 0..5 {
            let mut new_phases = best_phases;
            new_phases[index] += PHASE_ADJUSTMENT_VALUE;

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
     120, 140, 170, 170, 170, 170, 140, 120,
      80, 110, 140, 130, 130, 140, 110,  80,
      70, 100,  90, 100, 100,  90, 100,  70,
      60,  80,  80,  90,  90,  80,  80,  60,
      60,  90,  70,  80,  80,  70,  90,  60,
      60, 100,  80,  70,  70,  80, 100,  60,
       0,   0,   0,   0,   0,   0,   0,   0,


     190, 260, 270, 340, 340, 270, 260, 190,
     340, 330, 400, 370, 370, 400, 330, 340,
     350, 380, 410, 420, 420, 410, 380, 350,
     350, 350, 390, 390, 390, 390, 350, 350,
     330, 340, 350, 360, 360, 350, 340, 330,
     310, 330, 340, 350, 350, 340, 330, 310,
     300, 310, 320, 330, 330, 320, 310, 300,
     260, 300, 300, 300, 300, 300, 300, 260,


     320, 340, 320, 300, 300, 320, 340, 320,
     360, 380, 380, 370, 370, 380, 380, 360,
     380, 400, 410, 410, 410, 410, 400, 380,
     360, 380, 400, 410, 410, 400, 380, 360,
     360, 370, 380, 390, 390, 380, 370, 360,
     370, 370, 370, 380, 380, 370, 370, 370,
     370, 380, 380, 360, 360, 380, 380, 370,
     350, 370, 350, 340, 340, 350, 370, 350,


     540, 520, 530, 540, 540, 530, 520, 540,
     530, 520, 540, 530, 530, 540, 520, 530,
     510, 540, 520, 520, 520, 520, 540, 510,
     480, 490, 490, 490, 490, 490, 490, 480,
     460, 460, 460, 470, 470, 460, 460, 460,
     450, 470, 460, 460, 460, 460, 470, 450,
     440, 460, 460, 460, 460, 460, 460, 440,
     450, 470, 470, 470, 470, 470, 470, 450,


    1010,1050,1060,1070,1070,1060,1050,1010,
    1050,1020,1040,1020,1020,1040,1020,1050,
    1060,1060,1060,1050,1050,1060,1060,1060,
    1030,1030,1030,1020,1020,1030,1030,1030,
    1030,1020,1020,1020,1020,1020,1020,1030,
    1020,1030,1020,1020,1020,1020,1030,1020,
    1030,1030,1030,1030,1030,1030,1030,1030,
    1010,1010,1010,1020,1020,1010,1010,1010,


     -10,  20,   0, -30, -30,   0,  20, -10,
     -10, -10, -20,   0,   0, -20, -10, -10,
     -20,  20,   0, -20, -20,   0,  20, -20,
     -60, -50, -60, -70, -70, -60, -50, -60,
     -80, -70, -80,-100,-100, -80, -70, -80,
     -40, -20, -60, -70, -70, -60, -20, -40,
      30,  10, -10, -40, -40, -10,  10,  30,
      30,  50,   0, -30, -30,   0,  50,  30
    ];

    #[rustfmt::skip]
    let end_game_piece_square_tables: [i32; 384] = [
       0,   0,   0,   0,   0,   0,   0,   0,
     250, 240, 220, 200, 200, 220, 240, 250,
     190, 190, 150, 150, 150, 150, 190, 190,
     120, 120, 100, 100, 100, 100, 120, 120,
     100, 110,  90,  90,  90,  90, 110, 100,
     100, 100,  90, 100, 100,  90, 100, 100,
     100, 100, 100, 110, 110, 100, 100, 100,
       0,   0,   0,   0,   0,   0,   0,   0,


     210, 260, 270, 270, 270, 270, 260, 210,
     250, 270, 270, 280, 280, 270, 270, 250,
     260, 280, 290, 290, 290, 290, 280, 260,
     270, 300, 300, 310, 310, 300, 300, 270,
     270, 290, 300, 310, 310, 300, 290, 270,
     270, 280, 290, 300, 300, 290, 280, 270,
     260, 270, 280, 280, 280, 280, 270, 260,
     250, 240, 260, 270, 270, 260, 240, 250,


     290, 290, 300, 300, 300, 300, 290, 290,
     280, 290, 290, 300, 300, 290, 290, 280,
     300, 290, 300, 300, 300, 300, 290, 300,
     300, 310, 310, 310, 310, 310, 310, 300,
     290, 310, 310, 310, 310, 310, 310, 290,
     290, 290, 310, 310, 310, 310, 290, 290,
     280, 290, 290, 300, 300, 290, 290, 280,
     270, 280, 280, 290, 290, 280, 280, 270,


     520, 530, 540, 530, 530, 540, 530, 520,
     520, 530, 530, 530, 530, 530, 530, 520,
     520, 520, 520, 530, 530, 520, 520, 520,
     520, 520, 530, 530, 530, 530, 520, 520,
     520, 520, 530, 530, 530, 530, 520, 520,
     510, 510, 520, 520, 520, 520, 510, 510,
     510, 510, 510, 520, 520, 510, 510, 510,
     510, 520, 520, 520, 520, 520, 520, 510,


     950, 940, 960, 970, 970, 960, 940, 950,
     930, 970, 980,1010,1010, 980, 970, 930,
     930, 940, 980, 990, 990, 980, 940, 930,
     950, 970, 980,1000,1000, 980, 970, 950,
     940, 960, 970, 990, 990, 970, 960, 940,
     930, 930, 950, 960, 960, 950, 930, 930,
     890, 900, 910, 920, 920, 910, 900, 890,
     910, 910, 920, 910, 910, 920, 910, 910,


     -70, -20,   0,   0,   0,   0, -20, -70,
       0,  30,  30,  30,  30,  30,  30,   0,
      10,  30,  40,  50,  50,  40,  30,  10,
      10,  40,  50,  50,  50,  50,  40,  10,
       0,  20,  40,  50,  50,  40,  20,   0,
       0,  10,  20,  30,  30,  20,  10,   0,
     -20,   0,   0,  10,  10,   0,   0, -20,
     -50, -30, -10, -10, -10, -10, -30, -50,
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

    println!("Parsed dataset in {} seconds", time.elapsed().as_secs_f64());
    return;

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
