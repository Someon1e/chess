use core::str::SplitWhitespace;

#[derive(Default)]
pub struct SearchTimeInfo {
    pub white_time: Option<u128>,
    pub black_time: Option<u128>,
    pub white_increment: Option<u128>,
    pub black_increment: Option<u128>,
    pub moves_to_go: Option<u16>,
}

pub enum SearchTime {
    Infinite,
    Fixed(u128),
    Info(SearchTimeInfo),
}

pub enum SearchType {
    None,
    Perft,
}

pub struct GoParameters {
    pub nodes: Option<u64>,

    pub depth: Option<u16>,

    pub find_mate: Option<u16>,

    pub search_type: SearchType,

    pub move_time: Option<SearchTime>,
}

impl GoParameters {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            nodes: None,

            depth: None,

            find_mate: None,

            search_type: SearchType::None,

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
                        self.move_time = Some(SearchTime::Info(SearchTimeInfo::default()))
                    }

                    let move_time = match self.move_time {
                        Some(SearchTime::Info(ref mut info)) => info,
                        None => unreachable!(),
                        _ => panic!("Malformed input"),
                    };

                    match label {
                        "wtime" => {
                            assert!(move_time.white_time.is_none(), "Overwritten wtime");
                            move_time.white_time = parse_number!()
                        }
                        "btime" => {
                            assert!(move_time.black_time.is_none(), "Overwritten btime");
                            move_time.black_time = parse_number!()
                        }
                        "winc" => {
                            assert!(move_time.white_increment.is_none(), "Overwritten winc");
                            move_time.white_increment = parse_number!()
                        }
                        "binc" => {
                            assert!(move_time.black_increment.is_none(), "Overwritten binc");
                            move_time.black_increment = parse_number!()
                        }
                        "movestogo" => {
                            assert!(move_time.moves_to_go.is_none(), "Overwritten movestogo");
                            move_time.moves_to_go = parse_number!()
                        }

                        _ => unreachable!(),
                    };
                }

                "depth" => self.depth = parse_number!(),
                "nodes" => self.nodes = parse_number!(),
                "mate" => self.find_mate = parse_number!(),
                "movetime" => {
                    assert!(self.move_time.is_none(), "Malformed input");
                    self.move_time = Some(SearchTime::Fixed(parse_number!().unwrap()));
                }
                "perft" => {
                    self.search_type = SearchType::Perft;
                    self.depth = parse_number!();
                }
                "infinite" => {
                    assert!(self.move_time.is_none(), "Malformed input");
                    self.move_time = Some(SearchTime::Infinite);
                }
                _ => panic!("Unknown parameter"),
            }
        }
    }
}
