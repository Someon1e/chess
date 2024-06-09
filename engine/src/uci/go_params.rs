use core::str::SplitWhitespace;

#[derive(Default)]
pub struct MoveTimeInfo {
    pub white_time: Option<u128>,
    pub black_time: Option<u128>,
    pub white_increment: Option<u128>,
    pub black_increment: Option<u128>,
    pub moves_to_go: Option<i16>,
}

pub enum MoveTime {
    Infinite,
    Fixed(u128),
    Info(MoveTimeInfo),
}

pub struct GoParameters {
    pub nodes: Option<u64>,

    pub depth: Option<u16>,

    pub find_mate: Option<u16>,

    pub perft: bool,

    pub move_time: Option<MoveTime>,
}

impl GoParameters {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            nodes: None,

            depth: None,

            find_mate: None,

            perft: false,

            move_time: None,
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

                "wtime" | "btime" | "winc" | "binc" | "movestogo" => {
                    if self.move_time.is_none() {
                        self.move_time = Some(MoveTime::Info(MoveTimeInfo::default()))
                    }

                    let move_time = match self.move_time {
                        Some(MoveTime::Info(ref mut info)) => info,
                        None => unreachable!(),
                        _ => panic!("Malformed input"),
                    };

                    match label {
                        "wtime" => move_time.white_time = parse_number!(),
                        "btime" => move_time.black_time = parse_number!(),
                        "winc" => move_time.white_increment = parse_number!(),
                        "binc" => move_time.black_increment = parse_number!(),
                        "movestogo" => move_time.moves_to_go = parse_number!(),

                        _ => unreachable!()
                    };
                }

                "depth" => self.depth = parse_number!(),
                "nodes" => self.nodes = parse_number!(),
                "mate" => self.find_mate = parse_number!(),
                "movetime" => {
                    self.move_time = Some(MoveTime::Fixed(parse_number!().unwrap()));
                }
                "perft" => {
                    self.perft = true;
                    self.depth = parse_number!();
                }
                "infinite" => {
                    assert!(self.move_time.is_none(), "Malformed input");
                    self.move_time = Some(MoveTime::Infinite);
                }
                _ => panic!("Unknown parameter"),
            }
        }
    }
}
