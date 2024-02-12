use std::sync::OnceLock;

use crate::board::{
    bit_board::BitBoard,
    square::{Direction, Square, DIRECTIONS},
};

use super::precomputed::SQUARES_FROM_EDGE;

#[derive(Debug, Clone, Copy)]
struct Key {
    magic: u64,
    shift: u64,
}

fn iterate_combinations(squares: BitBoard) -> impl std::iter::Iterator<Item = BitBoard> {
    let mut next = Some(BitBoard::EMPTY);
    std::iter::from_fn(move || {
        let result = next;
        next = Some((next?.wrapping_sub(squares)) & squares);
        if next.unwrap() == BitBoard::EMPTY {
            next = None
        }
        result
    })
}

fn all_blockers(
    from: Square,
    directions: &[Direction],
    squares_from_edge: &[Direction],
) -> BitBoard {
    let mut bit_board = BitBoard::EMPTY;
    for (direction, distance_from_edge) in directions.iter().zip(squares_from_edge) {
        for count in 1..=*distance_from_edge - 1 {
            let move_to = from.offset(direction * count);
            bit_board.set(&move_to)
        }
    }
    bit_board
}

fn gen_slider_moves(
    from: Square,
    blockers: &BitBoard,
    direction_start_index: usize,
    direction_end_index: usize,
) -> BitBoard {
    let mut moves = BitBoard::EMPTY;

    let squares_from_edge = &SQUARES_FROM_EDGE[from.index() as usize];
    for index in direction_start_index..direction_end_index {
        let direction = DIRECTIONS[index];

        for count in 1..=squares_from_edge[index] {
            let move_to = from.offset(direction * count);

            moves.set(&move_to);
            if blockers.get(&move_to) {
                break;
            }
        }
    }
    moves
}

fn calculate_blockers_for_each_square(
    direction_start_index: usize,
    direction_end_index: usize,
) -> [BitBoard; 64] {
    let mut blockers = [BitBoard::EMPTY; 64];
    for square_index in 0..64 {
        blockers[square_index as usize] = all_blockers(
            Square::from_index(square_index),
            &DIRECTIONS[direction_start_index..direction_end_index],
            &SQUARES_FROM_EDGE[square_index as usize][direction_start_index..direction_end_index],
        )
    }
    blockers
}

pub fn relevant_rook_blockers() -> &'static [BitBoard; 64] {
    static COMPUTATION: OnceLock<[BitBoard; 64]> = OnceLock::new();
    COMPUTATION.get_or_init(|| calculate_blockers_for_each_square(0, 4))
}
pub fn relevant_bishop_blockers() -> &'static [BitBoard; 64] {
    static COMPUTATION: OnceLock<[BitBoard; 64]> = OnceLock::new();
    COMPUTATION.get_or_init(|| calculate_blockers_for_each_square(4, 8))
}

#[derive(Debug)]
struct TableFillError;
fn fill_magic_table(
    square: Square,
    magic: u64,
    index_bits: u64,
    direction_start_index: usize,
    direction_end_index: usize,
) -> Result<Vec<BitBoard>, TableFillError> {
    let mut table = vec![BitBoard::EMPTY; 1 << index_bits];
    let blockers = all_blockers(
        square,
        &DIRECTIONS[direction_start_index..direction_end_index],
        &SQUARES_FROM_EDGE[square.index() as usize][direction_start_index..direction_end_index],
    );
    for blocker_combination in iterate_combinations(blockers) {
        let moves = gen_slider_moves(
            square,
            &blocker_combination,
            direction_start_index,
            direction_end_index,
        );
        let table_entry = &mut table[magic_index(&blocker_combination, magic, 64 - index_bits)];
        if table_entry.is_empty() {
            *table_entry = moves;
        } else if *table_entry != moves {
            return Err(TableFillError);
        }
    }
    Ok(table)
}

fn magic_index(blockers: &BitBoard, magic: u64, shift: u64) -> usize {
    let hash = blockers.wrapping_mul(BitBoard::new(magic));
    (hash >> shift).as_usize()
}

const ROOK_KEYS: [Key; 64] = [
    Key {
        magic: 72075735996571841,
        shift: 52,
    },
    Key {
        magic: 18014674998071298,
        shift: 53,
    },
    Key {
        magic: 9295439526767886400,
        shift: 53,
    },
    Key {
        magic: 72093088789561600,
        shift: 53,
    },
    Key {
        magic: 11565252645623051264,
        shift: 53,
    },
    Key {
        magic: 144121785213813032,
        shift: 53,
    },
    Key {
        magic: 288239242105129090,
        shift: 53,
    },
    Key {
        magic: 108090927624487680,
        shift: 52,
    },
    Key {
        magic: 140772921843840,
        shift: 53,
    },
    Key {
        magic: 85357288029691904,
        shift: 54,
    },
    Key {
        magic: 4903012692341563394,
        shift: 54,
    },
    Key {
        magic: 9148039844331648,
        shift: 54,
    },
    Key {
        magic: 1153062791918135296,
        shift: 54,
    },
    Key {
        magic: 9225764591345074688,
        shift: 54,
    },
    Key {
        magic: 2306405963673383424,
        shift: 54,
    },
    Key {
        magic: 1153062245870077056,
        shift: 53,
    },
    Key {
        magic: 2323857957484265512,
        shift: 53,
    },
    Key {
        magic: 4611756662317916160,
        shift: 54,
    },
    Key {
        magic: 576480544083165313,
        shift: 54,
    },
    Key {
        magic: 4504149517406338,
        shift: 54,
    },
    Key {
        magic: 1153062791985234944,
        shift: 54,
    },
    Key {
        magic: 4620702018139340848,
        shift: 54,
    },
    Key {
        magic: 1152925902922459137,
        shift: 54,
    },
    Key {
        magic: 4917968176496542721,
        shift: 53,
    },
    Key {
        magic: 29343768471699456,
        shift: 53,
    },
    Key {
        magic: 4900162686261661696,
        shift: 54,
    },
    Key {
        magic: 52778707714176,
        shift: 54,
    },
    Key {
        magic: 17596481601826,
        shift: 54,
    },
    Key {
        magic: 144396736067534852,
        shift: 54,
    },
    Key {
        magic: 9819047856512827520,
        shift: 54,
    },
    Key {
        magic: 37172306325799424,
        shift: 54,
    },
    Key {
        magic: 10377420824348238849,
        shift: 53,
    },
    Key {
        magic: 864761497811681408,
        shift: 53,
    },
    Key {
        magic: 1152991942108258304,
        shift: 54,
    },
    Key {
        magic: 6917674300640079872,
        shift: 54,
    },
    Key {
        magic: 17594409027586,
        shift: 54,
    },
    Key {
        magic: 5651491939424257,
        shift: 54,
    },
    Key {
        magic: 1747537410096382464,
        shift: 54,
    },
    Key {
        magic: 45194330847053826,
        shift: 54,
    },
    Key {
        magic: 9234653032631767141,
        shift: 53,
    },
    Key {
        magic: 1548525762543632,
        shift: 53,
    },
    Key {
        magic: 148706749170335746,
        shift: 54,
    },
    Key {
        magic: 1154611179713331217,
        shift: 54,
    },
    Key {
        magic: 1162245440524124184,
        shift: 54,
    },
    Key {
        magic: 18577933118603268,
        shift: 54,
    },
    Key {
        magic: 4901042303084298316,
        shift: 54,
    },
    Key {
        magic: 576498135715577984,
        shift: 54,
    },
    Key {
        magic: 9259403583793791011,
        shift: 53,
    },
    Key {
        magic: 162129726171859072,
        shift: 53,
    },
    Key {
        magic: 35253129314880,
        shift: 54,
    },
    Key {
        magic: 9817864781464699264,
        shift: 54,
    },
    Key {
        magic: 177022581080320,
        shift: 54,
    },
    Key {
        magic: 1491008768508160,
        shift: 54,
    },
    Key {
        magic: 141845590179968,
        shift: 54,
    },
    Key {
        magic: 577023723799314944,
        shift: 54,
    },
    Key {
        magic: 72198335821529472,
        shift: 53,
    },
    Key {
        magic: 9234985082963918882,
        shift: 52,
    },
    Key {
        magic: 90213829822398469,
        shift: 53,
    },
    Key {
        magic: 1143638130296898,
        shift: 53,
    },
    Key {
        magic: 9044067522454785,
        shift: 53,
    },
    Key {
        magic: 151433642967567426,
        shift: 53,
    },
    Key {
        magic: 9800395756560454018,
        shift: 53,
    },
    Key {
        magic: 184731106853380,
        shift: 53,
    },
    Key {
        magic: 9799837188295237762,
        shift: 52,
    },
];

fn init_lookup(
    keys: [Key; 64],
    direction_start_index: usize,
    direction_end_index: usize,
) -> Vec<Vec<BitBoard>> {
    let mut lookup = vec![vec![]; 64];
    for square_index in 0..64 {
        let square = Square::from_index(square_index);

        let key = keys[square_index as usize];
        let filled = fill_magic_table(
            square,
            key.magic,
            64 - key.shift,
            direction_start_index,
            direction_end_index,
        )
        .unwrap();
        lookup[square_index as usize] = filled;
    }
    lookup
}

fn rook_lookup() -> &'static Vec<Vec<BitBoard>> {
    static COMPUTATION: OnceLock<Vec<Vec<BitBoard>>> = OnceLock::new();
    COMPUTATION.get_or_init(|| init_lookup(ROOK_KEYS, 0, 4))
}

pub fn get_rook_moves(square: Square, relevant_blockers: BitBoard) -> BitBoard {
    let key = ROOK_KEYS[square.index() as usize];
    rook_lookup()[square.index() as usize][magic_index(&relevant_blockers, key.magic, key.shift)]
}

const BISHOP_KEYS: [Key; 64] = [
    Key {
        magic: 10212833149092354,
        shift: 58,
    },
    Key {
        magic: 153124603818975264,
        shift: 59,
    },
    Key {
        magic: 4625201217639809256,
        shift: 59,
    },
    Key {
        magic: 2342505678981039120,
        shift: 59,
    },
    Key {
        magic: 6351205909989949952,
        shift: 59,
    },
    Key {
        magic: 9233648210505936920,
        shift: 59,
    },
    Key {
        magic: 162698086177316864,
        shift: 59,
    },
    Key {
        magic: 576533354501964802,
        shift: 58,
    },
    Key {
        magic: 4617051652619501632,
        shift: 59,
    },
    Key {
        magic: 4618725143525720320,
        shift: 59,
    },
    Key {
        magic: 1153502326997188612,
        shift: 59,
    },
    Key {
        magic: 4415368986624,
        shift: 59,
    },
    Key {
        magic: 9008372855636033,
        shift: 59,
    },
    Key {
        magic: 2305883708861054976,
        shift: 59,
    },
    Key {
        magic: 189156684189680138,
        shift: 59,
    },
    Key {
        magic: 292479564122112,
        shift: 59,
    },
    Key {
        magic: 2314850483651084896,
        shift: 59,
    },
    Key {
        magic: 2252487153493057,
        shift: 59,
    },
    Key {
        magic: 2326395114953515520,
        shift: 57,
    },
    Key {
        magic: 164381438509195264,
        shift: 57,
    },
    Key {
        magic: 2401488024371200,
        shift: 57,
    },
    Key {
        magic: 153194963709075457,
        shift: 57,
    },
    Key {
        magic: 76649713126868996,
        shift: 59,
    },
    Key {
        magic: 9223426260825801729,
        shift: 59,
    },
    Key {
        magic: 4512405401387346,
        shift: 59,
    },
    Key {
        magic: 9234798196589396737,
        shift: 59,
    },
    Key {
        magic: 577588859890641920,
        shift: 57,
    },
    Key {
        magic: 1153625742475657218,
        shift: 55,
    },
    Key {
        magic: 72339413702557696,
        shift: 55,
    },
    Key {
        magic: 211656140394496,
        shift: 57,
    },
    Key {
        magic: 99642485420132356,
        shift: 59,
    },
    Key {
        magic: 9871961061211637248,
        shift: 59,
    },
    Key {
        magic: 4756399393174884352,
        shift: 59,
    },
    Key {
        magic: 1182199480620958212,
        shift: 59,
    },
    Key {
        magic: 19275881382080,
        shift: 57,
    },
    Key {
        magic: 6757609204351104,
        shift: 55,
    },
    Key {
        magic: 90081892715528256,
        shift: 55,
    },
    Key {
        magic: 4800846017123591169,
        shift: 57,
    },
    Key {
        magic: 310794627866330144,
        shift: 59,
    },
    Key {
        magic: 4627449237745972736,
        shift: 59,
    },
    Key {
        magic: 9233650409527068736,
        shift: 59,
    },
    Key {
        magic: 577868711373840388,
        shift: 59,
    },
    Key {
        magic: 108368691346804736,
        shift: 57,
    },
    Key {
        magic: 1175439786219935744,
        shift: 57,
    },
    Key {
        magic: 1297041127538557952,
        shift: 57,
    },
    Key {
        magic: 9241409542307242241,
        shift: 57,
    },
    Key {
        magic: 2841172415488256,
        shift: 59,
    },
    Key {
        magic: 2884556218628575300,
        shift: 59,
    },
    Key {
        magic: 290284224643072,
        shift: 59,
    },
    Key {
        magic: 144609178102530080,
        shift: 59,
    },
    Key {
        magic: 2310346754887778305,
        shift: 59,
    },
    Key {
        magic: 6379348890459176964,
        shift: 59,
    },
    Key {
        magic: 13511967147163648,
        shift: 59,
    },
    Key {
        magic: 8878907064609,
        shift: 59,
    },
    Key {
        magic: 2891319825602199552,
        shift: 59,
    },
    Key {
        magic: 5638367098208288,
        shift: 59,
    },
    Key {
        magic: 1411775144919552,
        shift: 58,
    },
    Key {
        magic: 39424656062939648,
        shift: 59,
    },
    Key {
        magic: 2323861810133469444,
        shift: 59,
    },
    Key {
        magic: 9265349120,
        shift: 59,
    },
    Key {
        magic: 1686051619328,
        shift: 59,
    },
    Key {
        magic: 9007508762984962,
        shift: 59,
    },
    Key {
        magic: 11602416821675819072,
        shift: 59,
    },
    Key {
        magic: 6923316859299463680,
        shift: 58,
    },
];
fn bishop_lookup() -> &'static Vec<Vec<BitBoard>> {
    static COMPUTATION: OnceLock<Vec<Vec<BitBoard>>> = OnceLock::new();
    COMPUTATION.get_or_init(|| init_lookup(BISHOP_KEYS, 4, 8))
}

pub fn get_bishop_moves(square: Square, relevant_blockers: BitBoard) -> BitBoard {
    let key = BISHOP_KEYS[square.index() as usize];
    bishop_lookup()[square.index() as usize][magic_index(&relevant_blockers, key.magic, key.shift)]
}

#[cfg(test)]
mod tests {
    use crate::{
        board::{
            bit_board::BitBoard,
            square::{Square, DIRECTIONS},
        },
        move_generator::{
            precomputed::SQUARES_FROM_EDGE,
            slider_lookup::{
                get_bishop_moves, get_rook_moves, relevant_bishop_blockers, relevant_rook_blockers,
                Key,
            },
        },
    };

    use super::{all_blockers, fill_magic_table, iterate_combinations};

    use rand_chacha::rand_core::{RngCore, SeedableRng};

    #[test]
    fn find_rook_magics() {
        let mut random = rand_chacha::ChaCha20Rng::seed_from_u64(1);
        let mut rook_lookup: Vec<Vec<BitBoard>> = vec![vec![]; 64];
        let mut keys = [Key { magic: 0, shift: 0 }; 64];
        for square_index in 0..64 {
            let square = Square::from_index(square_index);

            loop {
                let magic = random.next_u64() & random.next_u64() & random.next_u64();
                let index_bits = all_blockers(
                    square,
                    &DIRECTIONS[0..4],
                    &SQUARES_FROM_EDGE[square.index() as usize][0..4],
                )
                .count() as u64;
                let filled = fill_magic_table(square, magic, index_bits, 0, 4);
                if let Ok(filled) = filled {
                    rook_lookup[square_index as usize] = filled;
                    keys[square_index as usize] = Key {
                        magic: magic,
                        shift: 64 - index_bits,
                    };
                    break;
                }
            }
        }
        println!("{keys:?}");
    }
    #[test]
    fn find_bishop_magics() {
        let mut random = rand_chacha::ChaCha20Rng::seed_from_u64(1);
        let mut bishop_lookup: Vec<Vec<BitBoard>> = vec![vec![]; 64];
        let mut keys = [Key { magic: 0, shift: 0 }; 64];
        for square_index in 0..64 {
            let square = Square::from_index(square_index);

            loop {
                let magic = random.next_u64() & random.next_u64() & random.next_u64();
                let index_bits = all_blockers(
                    square,
                    &DIRECTIONS[4..8],
                    &SQUARES_FROM_EDGE[square.index() as usize][4..8],
                )
                .count() as u64;
                let filled = fill_magic_table(square, magic, index_bits, 4, 8);
                if let Ok(filled) = filled {
                    bishop_lookup[square_index as usize] = filled;
                    keys[square_index as usize] = Key {
                        magic: magic,
                        shift: 64 - index_bits,
                    };
                    break;
                }
            }
        }
        println!("{keys:?}");
    }

    #[test]
    fn blocker_combinations() {
        let d4 = Square::from_notation("d4");
        let blockers = all_blockers(d4, &DIRECTIONS, &SQUARES_FROM_EDGE[d4.index() as usize]);
        let expected_number_of_combinations = 1 << blockers.count();
        println!("{blockers}");
        let mut number_of_combinations = 0;
        for _ in iterate_combinations(blockers) {
            number_of_combinations += 1;
        }
        assert_eq!(number_of_combinations, expected_number_of_combinations)
    }

    #[test]
    fn move_lookup() {
        let d4 = Square::from_notation("d4");
        let mut blockers = BitBoard::EMPTY;
        blockers.set(&Square::from_notation("f4"));

        let rook_moves =
            get_rook_moves(d4, blockers & relevant_rook_blockers()[d4.index() as usize]);
        let bishop_moves = get_bishop_moves(
            d4,
            blockers & relevant_bishop_blockers()[d4.index() as usize],
        );

        let legal_moves = rook_moves | bishop_moves;
        println!("{}", legal_moves);
        assert_eq!(legal_moves.count(), 25)
    }
}
