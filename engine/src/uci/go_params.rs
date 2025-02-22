use core::num::{NonZeroU16, NonZeroU64};
use core::str::SplitWhitespace;

use crate::search::Ply;

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
    nodes: Option<u64>,

    depth: Option<u16>,

    mate_in_moves: Option<u8>,

    pondering: Option<bool>,

    search_type: SearchType,

    move_time: Option<SearchTime>,
}

impl GoParameters {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            nodes: None,
            depth: None,
            mate_in_moves: None,
            search_type: SearchType::None,
            move_time: None,
            pondering: None,
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
                "ponder" => {
                    assert!(self.pondering.is_none(), "Pondering defined twice");
                    self.pondering = Some(true);
                }

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
                    }
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
                    assert!(self.mate_in_moves.is_none(), "Conflicting mate");
                    let mate_in_moves = parse_number!().unwrap();
                    const MAX_MOVES: u32 = (Ply::MAX as u32 + 1) / 2;
                    assert!(mate_in_moves < MAX_MOVES as Ply);
                    self.mate_in_moves = Some(mate_in_moves);
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
    #[must_use]
    pub const fn search_type(&self) -> &SearchType {
        &self.search_type
    }
    #[must_use]
    pub const fn move_time(self) -> Option<SearchTime> {
        self.move_time
    }
    #[must_use]
    pub const fn mate_in_moves(&self) -> Option<u8> {
        self.mate_in_moves
    }
    #[must_use]
    pub const fn depth(&self) -> Option<u16> {
        self.depth
    }
    #[must_use]
    pub const fn nodes(&self) -> Option<u64> {
        self.nodes
    }
    #[must_use]
    pub const fn pondering(&self) -> Option<bool> {
        self.pondering
    }
}
