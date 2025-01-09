use core::str::SplitWhitespace;
use std::num::{NonZeroU16, NonZeroU64};

#[derive(Default)]
pub struct SearchTimeInfo {
    pub white_time: Option<u64>,
    pub black_time: Option<u64>,
    pub white_increment: Option<NonZeroU64>,
    pub black_increment: Option<NonZeroU64>,
    pub moves_to_go: Option<NonZeroU16>,
}

pub enum SearchTime {
    Infinite,
    Fixed(u64),
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
                    args.next().unwrap().parse()
                };
            }
            match label {
                "searchmoves" => todo!(),
                "ponder" => todo!("Pondering"),

                "wtime" | "btime" | "winc" | "binc" | "movestogo" => {
                    if self.move_time.is_none() {
                        self.move_time = Some(SearchTime::Info(SearchTimeInfo::default()));
                    }

                    let move_time = match self.move_time {
                        Some(SearchTime::Info(ref mut info)) => info,
                        None => unreachable!(),
                        _ => panic!("Conflicting move time"),
                    };

                    match label {
                        "wtime" => {
                            assert!(move_time.white_time.is_none(), "Overwritten wtime");
                            move_time.white_time = Some(parse_number!().unwrap());
                        }
                        "btime" => {
                            assert!(move_time.black_time.is_none(), "Overwritten btime");
                            move_time.black_time = Some(parse_number!().unwrap());
                        }
                        "winc" => {
                            assert!(move_time.white_increment.is_none(), "Overwritten winc");
                            if let Ok(winc) = parse_number!() {
                                move_time.white_increment = NonZeroU64::new(winc);
                            } else {
                                move_time.white_increment = None;
                            }
                        }
                        "binc" => {
                            assert!(move_time.black_increment.is_none(), "Overwritten binc");
                            if let Ok(binc) = parse_number!() {
                                move_time.black_increment = NonZeroU64::new(binc);
                            } else {
                                move_time.black_increment = None;
                            }
                        }
                        "movestogo" => {
                            assert!(move_time.moves_to_go.is_none(), "Overwritten movestogo");
                            move_time.moves_to_go = Some(parse_number!().unwrap());
                        }

                        _ => unreachable!(),
                    };
                }

                "depth" => {
                    assert!(self.depth.is_none(), "Conflicting depth");
                    self.depth = Some(parse_number!().unwrap());
                }
                "nodes" => {
                    assert!(self.nodes.is_none(), "Conflicting nodes");
                    self.nodes = Some(parse_number!().unwrap());
                }
                "mate" => {
                    assert!(self.find_mate.is_none(), "Conflicting mate");
                    self.find_mate = Some(parse_number!().unwrap());
                }
                "movetime" => {
                    assert!(self.move_time.is_none(), "Conflicting move time");
                    self.move_time = Some(SearchTime::Fixed(parse_number!().unwrap()));
                }
                "perft" => {
                    self.search_type = SearchType::Perft;
                    self.depth = Some(parse_number!().unwrap());
                }
                "infinite" => {
                    assert!(self.move_time.is_none(), "Conflicting move time");
                    self.move_time = Some(SearchTime::Infinite);
                }
                _ => panic!("Unknown parameter"),
            }
        }
    }
}
