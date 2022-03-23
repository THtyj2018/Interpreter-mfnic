//! Grammer Parser

use crate::lexer::Token;

#[cfg(feature = "enable_log")]
use log;

pub(crate) enum ASTNode {
    Inner(u32, Vec<ASTNode>),
    Leaf(Token),
}

impl ToString for ASTNode {
    fn to_string(&self) -> String {
        self.to_string_impl(0)
    }
}

impl ASTNode {
    fn to_string_impl(&self, level: usize) -> String {
        match self {
            ASTNode::Inner(id, children) => {
                let indents = "|   ".repeat(level);
                let children_fmt: String = children
                    .iter()
                    .map(|n| format!("{}|---{},\n", indents, n.to_string_impl(level + 1)))
                    .collect();
                format!(
                    "Inner(\"{}\", [\n{}{}])",
                    Parser::GRAMMER[*id as usize],
                    children_fmt,
                    indents
                )
            }
            ASTNode::Leaf(token) => format!("Leaf({})", token.to_string()),
        }
    }

    pub(crate) fn assume_leaf(self) -> Token {
        match self {
            ASTNode::Leaf(token) => token,
            _ => panic!("Can't unwrap an ast leaf node"),
        }
    }
}

pub(crate) struct Parser {
    stack: Vec<u32>,
    top: u32,
    nodes: Vec<ASTNode>,
}

impl Parser {
    pub(crate) fn new() -> Self {
        Parser {
            stack: vec![],
            top: 0,
            nodes: vec![],
        }
    }

    const GRAMMER: &'static [&'static str; 22] = &[
        "",
        "S -> A",
        "S -> E",
        "A -> i=E",
        "A -> i:V=E",
        "V -> V,i",
        "V -> i",
        "E -> (E)",
        "E -> !E",
        "E -> pE",
        "E -> E^E",
        "E -> EmE",
        "E -> EpE",
        "E -> EcE",
        "E -> EoE",
        "E -> EaE",
        "E -> E?E:E",
        "E -> i(P)",
        "E -> i",
        "E -> n",
        "P -> P,i",
        "P -> i",
    ];

    //   i  n  =  (  )  !  ^  m  p  c  o  a  ?  :  ,
    const ACTION: &'static [[i32; Token::COUNT]; 44] = &[
        [3, 4, 0, 5, 0, 6, 0, 0, 7, 0, 0, 0, 0, 0, 0],
        [0; Token::COUNT],
        [0, 0, 0, 0, 0, 0, 8, 9, 10, 11, 12, 13, 14, 0, 0],
        [0, 0, 15, 16, 0, 0, -18, -18, -18, -18, -18, -18, -18, 17, 0],
        [
            0, 0, 0, 0, -19, 0, -19, -19, -19, -19, -19, -19, -19, -19, -19,
        ],
        [19, 4, 0, 5, 0, 6, 0, 0, 7, 0, 0, 0, 0, 0, 0],
        [19, 4, 0, 5, 0, 6, 0, 0, 7, 0, 0, 0, 0, 0, 0],
        [19, 4, 0, 5, 0, 6, 0, 0, 7, 0, 0, 0, 0, 0, 0],
        [19, 4, 0, 5, 0, 6, 0, 0, 7, 0, 0, 0, 0, 0, 0],
        [19, 4, 0, 5, 0, 6, 0, 0, 7, 0, 0, 0, 0, 0, 0],
        [19, 4, 0, 5, 0, 6, 0, 0, 7, 0, 0, 0, 0, 0, 0],
        [19, 4, 0, 5, 0, 6, 0, 0, 7, 0, 0, 0, 0, 0, 0],
        [19, 4, 0, 5, 0, 6, 0, 0, 7, 0, 0, 0, 0, 0, 0],
        [19, 4, 0, 5, 0, 6, 0, 0, 7, 0, 0, 0, 0, 0, 0],
        [19, 4, 0, 5, 0, 6, 0, 0, 7, 0, 0, 0, 0, 0, 0],
        [19, 4, 0, 5, 0, 6, 0, 0, 7, 0, 0, 0, 0, 0, 0],
        [19, 4, 0, 5, 0, 6, 0, 0, 7, 0, 0, 0, 0, 0, 0],
        [33, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 34, 0, 8, 9, 10, 11, 12, 13, 14, 0, 0],
        [
            0, 0, 0, 16, -18, 0, -18, -18, -18, -18, -18, -18, -18, -18, -18,
        ],
        [0, 0, 0, 0, -8, 0, -8, -8, -8, -8, -8, -8, -8, -8, -8],
        [0, 0, 0, 0, -9, 0, -9, -9, -9, -9, -9, -9, -9, -9, -9],
        [
            0, 0, 0, 0, -10, 0, -10, -10, -10, -10, -10, -10, -10, -10, -10,
        ],
        [
            0, 0, 0, 0, -11, 0, 8, -11, -11, -11, -11, -11, -11, -11, -11,
        ],
        [0, 0, 0, 0, -12, 0, 8, 9, -12, -12, -12, -12, -12, -12, -12],
        [0, 0, 0, 0, -13, 0, 8, 9, 10, -13, -13, -13, -13, -13, -13],
        [0, 0, 0, 0, -14, 0, 8, 9, 10, 11, -14, -14, -14, -14, -14],
        [0, 0, 0, 0, -15, 0, 8, 9, 10, 11, 12, -15, -15, -15, -15],
        [0, 0, 0, 0, 0, 0, 8, 9, 10, 11, 12, 13, 14, 35, 0],
        [0, 0, 0, 0, 0, 0, 8, 9, 10, 11, 12, 13, 14, 0, 0],
        [0, 0, 0, 0, -21, 0, 8, 9, 10, 11, 12, 13, 14, 0, -21],
        [0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 37],
        [0, 0, 38, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 39],
        [0, 0, -6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -6],
        [0, 0, 0, 0, -7, 0, -7, -7, -7, -7, -7, -7, -7, -7, -7],
        [19, 4, 0, 5, 0, 6, 0, 0, 7, 0, 0, 0, 0, 0, 0],
        [
            0, 0, 0, 0, -17, 0, -17, -17, -17, -17, -17, -17, -17, -17, -17,
        ],
        [19, 4, 0, 5, 0, 6, 0, 0, 7, 0, 0, 0, 0, 0, 0],
        [19, 4, 0, 5, 0, 6, 0, 0, 7, 0, 0, 0, 0, 0, 0],
        [43, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, -16, 0, 8, 9, 10, 11, 12, 13, 14, -16, -16],
        [0, 0, 0, 0, -20, 0, 8, 9, 10, 11, 12, 13, 14, 0, -20],
        [0, 0, 0, 0, 0, 0, 8, 9, 10, 11, 12, 13, 14, 0, 0],
        [0, 0, -5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -5],
    ];

    pub(crate) fn action(&mut self, token: Token) -> bool {
        let act = Self::ACTION[self.top as usize][token.id() as usize];
        self.stack.push(self.top);
        let state = if act > 0 {
            act as u32
        } else if act < 0 {
            self.reduce(-act as u32)
        } else {
            return false;
        };
        self.top = state;
        if act < 0 {
            #[cfg(feature = "enable_log")]
            log::info!(
                "Token {}; Reduce {}; Goto {}; Stack = {:?}",
                token.to_string(),
                -act,
                self.top,
                self.stack
            );
            self.action(token)
        } else {
            #[cfg(feature = "enable_log")]
            log::info!(
                "Token {}; Shift {}; Stack = {:?}",
                token.to_string(),
                self.top,
                self.stack
            );
            self.nodes.push(ASTNode::Leaf(token));
            return true;
        }
    }

    pub(crate) fn accept(mut self) -> Option<ASTNode> {
        let reduce = match self.top {
            1 | 2 => return Some(ASTNode::Inner(self.top, self.nodes)),
            3 => 18,
            4 => 19,
            19 => 18,
            20 => 8,
            21 => 9,
            22 => 10,
            23 => 11,
            24 => 12,
            25 => 13,
            26 => 14,
            27 => 15,
            29 => 3,
            30 => 21,
            34 => 7,
            36 => 17,
            40 => 16,
            42 => 4,
            _ => return None,
        };
        self.stack.push(self.top);
        self.top = self.reduce(reduce);
        #[cfg(feature = "enable_log")]
        log::info!(
            "Accepting; Reduce {}; Goto {}; Stack = {:?}",
            reduce,
            self.top,
            self.stack
        );
        self.accept()
    }

    fn reduce(&mut self, id: u32) -> u32 {
        let len = match id {
            6 | 18 | 19 | 21 => 1,
            8 | 9 => 2,
            3 | 5 | 7 | 10 | 11 | 12 | 13 | 14 | 15 | 20 => 3,
            17 => 4,
            4 | 16 => 5,
            _ => unreachable!(),
        };
        self.stack.truncate(self.stack.len() - len);
        let node = ASTNode::Inner(id, self.nodes.drain((self.nodes.len() - len)..).collect());
        self.nodes.push(node);
        let k = *self.stack.last().unwrap();
        if id >= 7 && id < 20 {
            if k >= 6 && k < 17 {
                k + 14
            } else {
                match k {
                    0 => 2,
                    5 => 18,
                    35 => 40,
                    37 => 41,
                    38 => 42,
                    _ => unreachable!(),
                }
            }
        } else {
            match id {
                3 | 4 => match k {
                    0 => 1,
                    _ => unreachable!(),
                },
                5 | 6 => match k {
                    17 => 32,
                    _ => unreachable!(),
                },
                20 | 21 => match k {
                    16 => 31,
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            }
        }
    }
}
