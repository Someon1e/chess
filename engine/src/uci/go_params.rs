use core::str::SplitWhitespace;

pub struct GoParameters {
    pub white_time: Option<u128>,
    pub black_time: Option<u128>,

    pub white_increment: Option<u128>,
    pub black_increment: Option<u128>,

    pub move_time_in_ms: Option<u128>,

    pub nodes: Option<u64>,

    pub moves_to_go: Option<u16>,

    pub depth: Option<u16>,

    pub find_mate: Option<u16>,

    pub perft: bool,

    pub infinite: bool,
}

impl GoParameters {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            white_time: None,
            black_time: None,

            white_increment: None,
            black_increment: None,

            move_time_in_ms: None,

            nodes: None,

            moves_to_go: None,

            depth: None,

            find_mate: None,

            perft: false,

            infinite: false,
        }
    }
    pub fn parse(&mut self, args: &mut SplitWhitespace) {
        while let Some(label) = args.next() {
            macro_rules! parse_number {
                () => {
                    Some(args.next().unwrap().parse().unwrap())
                };
            }
            match label {
                "searchmoves" => todo!(),
                "ponder" => todo!("Pondering"),
                "wtime" => self.white_time = parse_number!(),
                "btime" => self.black_time = parse_number!(),
                "winc" => self.white_increment = parse_number!(),
                "binc" => self.black_increment = parse_number!(),
                "movestogo" => self.moves_to_go = parse_number!(),
                "depth" => self.depth = parse_number!(),
                "nodes" => self.nodes = parse_number!(),
                "mate" => self.find_mate = parse_number!(),
                "movetime" => self.move_time_in_ms = parse_number!(),
                "perft" => {
                    self.perft = true;
                    self.depth = parse_number!();
                }
                "infinite" => {
                    self.infinite = true;
                }
                _ => panic!("Unknown parameter"),
            }
        }
    }
}
