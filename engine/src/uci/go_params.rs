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
            match label {
                "searchmoves" => {}
                "ponder" => {}
                "wtime" => {
                    self.white_time = Some(args.next().unwrap().parse().unwrap());
                }
                "btime" => {
                    self.black_time = Some(args.next().unwrap().parse().unwrap());
                }
                "winc" => {
                    self.white_increment = Some(args.next().unwrap().parse().unwrap());
                }
                "binc" => {
                    self.black_increment = Some(args.next().unwrap().parse().unwrap());
                }
                "movestogo" => {
                    self.moves_to_go = Some(args.next().unwrap().parse().unwrap());
                }
                "depth" => {
                    self.depth = Some(args.next().unwrap().parse().unwrap());
                }
                "nodes" => {
                    self.nodes = Some(args.next().unwrap().parse().unwrap());
                }
                "mate" => {
                    self.find_mate = Some(args.next().unwrap().parse().unwrap());
                }
                "movetime" => {
                    self.move_time_in_ms = Some(args.next().unwrap().parse().unwrap());
                }
                "perft" => {
                    self.perft = true;
                    self.depth = Some(args.next().unwrap().parse().unwrap());
                }
                "infinite" => {
                    self.infinite = true;
                }
                _ => unimplemented!(),
            }
        }
    }
}
