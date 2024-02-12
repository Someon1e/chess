use std::sync::OnceLock;

use crate::board::{
    bit_board::BitBoard,
    square::{Direction, Square, DIRECTIONS},
};

use super::precomputed::SQUARES_FROM_EDGE;

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

const ROOK_MAGICS: [u64; 64] = [
    72075735996571841,
    18014674998071298,
    9295439526767886400,
    72093088789561600,
    11565252645623051264,
    144121785213813032,
    288239242105129090,
    108090927624487680,
    140772921843840,
    85357288029691904,
    4903012692341563394,
    9148039844331648,
    1153062791918135296,
    9225764591345074688,
    2306405963673383424,
    1153062245870077056,
    2323857957484265512,
    4611756662317916160,
    576480544083165313,
    4504149517406338,
    1153062791985234944,
    4620702018139340848,
    1152925902922459137,
    4917968176496542721,
    29343768471699456,
    4900162686261661696,
    52778707714176,
    17596481601826,
    144396736067534852,
    9819047856512827520,
    37172306325799424,
    10377420824348238849,
    864761497811681408,
    1152991942108258304,
    6917674300640079872,
    17594409027586,
    5651491939424257,
    1747537410096382464,
    45194330847053826,
    9234653032631767141,
    1548525762543632,
    148706749170335746,
    1154611179713331217,
    1162245440524124184,
    18577933118603268,
    4901042303084298316,
    576498135715577984,
    9259403583793791011,
    162129726171859072,
    35253129314880,
    9817864781464699264,
    177022581080320,
    1491008768508160,
    141845590179968,
    577023723799314944,
    72198335821529472,
    9234985082963918882,
    90213829822398469,
    1143638130296898,
    9044067522454785,
    151433642967567426,
    9800395756560454018,
    184731106853380,
    9799837188295237762,
];
const ROOK_SHIFTS: [u64; 64] = [
    52, 53, 53, 53, 53, 53, 53, 52, 53, 54, 54, 54, 54, 54, 54, 53, 53, 54, 54, 54, 54, 54, 54, 53,
    53, 54, 54, 54, 54, 54, 54, 53, 53, 54, 54, 54, 54, 54, 54, 53, 53, 54, 54, 54, 54, 54, 54, 53,
    53, 54, 54, 54, 54, 54, 54, 53, 52, 53, 53, 53, 53, 53, 53, 52,
];

fn init_lookup(
    magics: [u64; 64],
    shifts: [u64; 64],
    direction_start_index: usize,
    direction_end_index: usize,
) -> Vec<Vec<BitBoard>> {
    let mut lookup = vec![vec![]; 64];
    for square_index in 0..64 {
        let square = Square::from_index(square_index);

        let magic = magics[square_index as usize];
        let shift = shifts[square_index as usize];
        let filled = fill_magic_table(
            square,
            magic,
            64 - shift,
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
    COMPUTATION.get_or_init(|| init_lookup(ROOK_MAGICS, ROOK_SHIFTS, 0, 4))
}

pub fn get_rook_moves(square: Square, relevant_blockers: BitBoard) -> BitBoard {
    let magic = ROOK_MAGICS[square.index() as usize];
    let shift = ROOK_SHIFTS[square.index() as usize];
    rook_lookup()[square.index() as usize][magic_index(&relevant_blockers, magic, shift)]
}

const BISHOP_MAGICS: [u64; 64] = [
    10212833149092354,
    153124603818975264,
    4625201217639809256,
    2342505678981039120,
    6351205909989949952,
    9233648210505936920,
    162698086177316864,
    576533354501964802,
    4617051652619501632,
    4618725143525720320,
    1153502326997188612,
    4415368986624,
    9008372855636033,
    2305883708861054976,
    189156684189680138,
    292479564122112,
    2314850483651084896,
    2252487153493057,
    2326395114953515520,
    164381438509195264,
    2401488024371200,
    153194963709075457,
    76649713126868996,
    9223426260825801729,
    4512405401387346,
    9234798196589396737,
    577588859890641920,
    1153625742475657218,
    72339413702557696,
    211656140394496,
    99642485420132356,
    9871961061211637248,
    4756399393174884352,
    1182199480620958212,
    19275881382080,
    6757609204351104,
    90081892715528256,
    4800846017123591169,
    310794627866330144,
    4627449237745972736,
    9233650409527068736,
    577868711373840388,
    108368691346804736,
    1175439786219935744,
    1297041127538557952,
    9241409542307242241,
    2841172415488256,
    2884556218628575300,
    290284224643072,
    144609178102530080,
    2310346754887778305,
    6379348890459176964,
    13511967147163648,
    8878907064609,
    2891319825602199552,
    5638367098208288,
    1411775144919552,
    39424656062939648,
    2323861810133469444,
    9265349120,
    1686051619328,
    9007508762984962,
    11602416821675819072,
    6923316859299463680,
];
const BISHOP_SHIFTS: [u64; 64] = [
    58, 59, 59, 59, 59, 59, 59, 58, 59, 59, 59, 59, 59, 59, 59, 59, 59, 59, 57, 57, 57, 57, 59, 59,
    59, 59, 57, 55, 55, 57, 59, 59, 59, 59, 57, 55, 55, 57, 59, 59, 59, 59, 57, 57, 57, 57, 59, 59,
    59, 59, 59, 59, 59, 59, 59, 59, 58, 59, 59, 59, 59, 59, 59, 58,
];

fn bishop_lookup() -> &'static Vec<Vec<BitBoard>> {
    static COMPUTATION: OnceLock<Vec<Vec<BitBoard>>> = OnceLock::new();
    COMPUTATION.get_or_init(|| init_lookup(BISHOP_MAGICS, BISHOP_SHIFTS, 4, 8))
}

pub fn get_bishop_moves(square: Square, relevant_blockers: BitBoard) -> BitBoard {
    let magic = BISHOP_MAGICS[square.index() as usize];
    let shift = BISHOP_SHIFTS[square.index() as usize];
    bishop_lookup()[square.index() as usize][magic_index(&relevant_blockers, magic, shift)]
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
            },
        },
    };

    use super::{all_blockers, fill_magic_table, iterate_combinations};

    use rand_chacha::rand_core::{RngCore, SeedableRng};

    #[test]
    fn find_rook_magics() {
        let mut random = rand_chacha::ChaCha20Rng::seed_from_u64(1);
        let mut rook_lookup: Vec<Vec<BitBoard>> = vec![vec![]; 64];
        let mut rook_shifts = [0; 64];
        let mut magics = [0; 64];
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
                    rook_shifts[square_index as usize] = 64 - index_bits;
                    magics[square_index as usize] = magic;
                    break;
                }
            }
        }
        println!("{magics:?} {rook_shifts:?}");
    }
    #[test]
    fn find_bishop_magics() {
        let mut random = rand_chacha::ChaCha20Rng::seed_from_u64(1);
        let mut bishop_lookup: Vec<Vec<BitBoard>> = vec![vec![]; 64];
        let mut bishop_shifts = [0; 64];
        let mut magics = [0; 64];
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
                    bishop_shifts[square_index as usize] = 64 - index_bits;
                    magics[square_index as usize] = magic;
                    break;
                }
            }
        }
        println!("{magics:?} {bishop_shifts:?}");
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
