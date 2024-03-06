use std::str::SplitWhitespace;

#[derive(Default)]
pub struct GoParameters {
    pub white_time: Option<u128>,
    pub black_time: Option<u128>,
    pub white_increment: Option<u128>,
    pub black_increment: Option<u128>,
    pub moves_to_go: Option<u64>,
    pub perft: bool,
    pub depth: Option<u16>,
    pub infinite: bool,
    pub nodes: Option<u64>,
    pub find_mate: Option<u64>,
    pub move_time_in_ms: Option<u128>,
}
impl GoParameters {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            white_time: None,
            black_time: None,
            white_increment: None,
            black_increment: None,
            moves_to_go: None,
            perft: false,
            depth: None,
            infinite: false,
            nodes: None,
            find_mate: None,
            move_time_in_ms: None,
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
                "searchmoves" => {}
                "ponder" => {}
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
                _ => unimplemented!(),
            }
        }
    }
}
