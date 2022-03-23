//! The lexer

use crate::Real;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MulDivOp {
    MUL,
    DIV,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AddSubOp {
    ADD,
    SUB,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CompareOp {
    LT,
    GT,
    LE,
    GE,
    EQ,
    NE,
    CMP,
}

impl CompareOp {
    pub(crate) fn on(self, r1: Real, r2: Real) -> Real {
        if r1 > r2 {
            match self {
                CompareOp::GT | CompareOp::GE | CompareOp::NE | CompareOp::CMP => 1.0,
                CompareOp::LT | CompareOp::LE | CompareOp::EQ => 0.0,
            }
        } else if r1 < r2 {
            match self {
                CompareOp::LT | CompareOp::LE | CompareOp::NE => 1.0,
                CompareOp::GT | CompareOp::GE | CompareOp::EQ => 0.0,
                CompareOp::CMP => -1.0,
            }
        } else {
            match self {
                CompareOp::GE | CompareOp::LE | CompareOp::EQ => 1.0,
                CompareOp::GT | CompareOp::LT | CompareOp::NE | CompareOp::CMP => 0.0,
            }
        }
    }
}

pub(crate) type Ident = Vec<u8>;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Token {
    IDENT(Ident),
    NUM(Real),
    ASSIGN,
    LPAREN,
    RPAREN,
    NOT,
    EXP,
    MD(MulDivOp),
    PN(AddSubOp),
    CMP(CompareOp),
    OR,
    AND,
    COND,
    COLON,
    COMMA,
}

impl Token {
    pub(crate) const COUNT: usize = 15;

    pub(crate) const fn id(&self) -> u32 {
        match self {
            Token::IDENT(_) => 0,
            Token::NUM(_) => 1,
            Token::ASSIGN => 2,
            Token::LPAREN => 3,
            Token::RPAREN => 4,
            Token::NOT => 5,
            Token::EXP => 6,
            Token::MD(_) => 7,
            Token::PN(_) => 8,
            Token::CMP(_) => 9,
            Token::OR => 10,
            Token::AND => 11,
            Token::COND => 12,
            Token::COLON => 13,
            Token::COMMA => 14,
        }
    }

    pub(crate) fn assume_ident(self) -> Ident {
        match self {
            Token::IDENT(ident) => ident,
            _ => panic!("Can't unwrap an ident"),
        }
    }

    pub(crate) fn assume_num(self) -> Real {
        match self {
            Token::NUM(num) => num,
            _ => panic!("Can't unwrap an ident"),
        }
    }

    pub(crate) fn assume_md(self) -> MulDivOp {
        match self {
            Token::MD(md) => md,
            _ => panic!("Can't unwrap mul or div assign"),
        }
    }

    pub(crate) fn assume_pn(self) -> AddSubOp {
        match self {
            Token::PN(pn) => pn,
            _ => panic!("Can't unwrap add or sub sign"),
        }
    }

    pub(crate) fn assume_cmp(self) -> CompareOp {
        match self {
            Token::CMP(cmp) => cmp,
            _ => panic!("Can't unwrap add or sub sign"),
        }
    }
}

impl ToString for Token {
    fn to_string(&self) -> String {
        match self {
            Token::IDENT(ident) => {
                format!("IDENT(\"{}\")", String::from_utf8(ident.clone()).unwrap())
            }
            _ => format!("{:?}", self),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InvalidToken {
    column: usize,
    expect: &'static str,
    found: String,
}

pub(crate) struct Lexer<'a> {
    line: &'a [u8],
    column: usize,
    begin: usize,
    stream: TokenStream,
}

pub(crate) struct TokenStream {
    pub(crate) complete: bool,
    pub(crate) tokens: Vec<(usize, Token)>,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(line: &'a [u8]) -> Self {
        Lexer {
            line,
            column: 0,
            begin: 0,
            stream: TokenStream {
                complete: true,
                tokens: vec![],
            },
        }
    }

    pub(crate) fn tokenize(mut self) -> Result<TokenStream, InvalidToken> {
        loop {
            let c = self.skip_whitespace();
            if c.is_ascii_alphabetic() || c == b'_' {
                self.eat();
                while self.cur().is_ascii_alphanumeric() || self.cur() == b'_' {
                    self.eat();
                }
                self.push(Token::IDENT(self.line[self.begin..self.column].to_vec()));
            } else if c.is_ascii_digit() {
                self.read_number()?;
            } else {
                self.eat();
                match c {
                    b'=' => {
                        if self.cur() == b'=' {
                            self.eat();
                            self.push(Token::CMP(CompareOp::EQ));
                        } else {
                            self.push(Token::ASSIGN);
                        }
                    }
                    b'!' => {
                        if self.cur() == b'=' {
                            self.eat();
                            self.push(Token::CMP(CompareOp::NE));
                        } else {
                            self.push(Token::NOT);
                        }
                    }
                    b'>' => {
                        if self.cur() == b'=' {
                            self.eat();
                            self.push(Token::CMP(CompareOp::GE));
                        } else {
                            self.push(Token::CMP(CompareOp::GT));
                        }
                    }
                    b'<' => {
                        if self.cur() == b'=' {
                            self.eat();
                            if self.cur() == b'>' {
                                self.eat();
                                self.push(Token::CMP(CompareOp::CMP));
                            }
                            self.push(Token::CMP(CompareOp::LE));
                        } else {
                            self.push(Token::CMP(CompareOp::LT))
                        }
                    }
                    b'|' => {
                        if self.cur() == b'|' {
                            self.eat();
                            self.push(Token::OR);
                        } else {
                            return self.err("logical 'or' operator");
                        }
                    }
                    b'&' => {
                        if self.cur() == b'&' {
                            self.eat();
                            self.push(Token::AND);
                        } else {
                            return self.err("logical 'and' operator");
                        }
                    }
                    b'(' => self.push(Token::LPAREN),
                    b')' => self.push(Token::RPAREN),
                    b'^' => self.push(Token::EXP),
                    b'*' => self.push(Token::MD(MulDivOp::MUL)),
                    b'/' => self.push(Token::MD(MulDivOp::DIV)),
                    b'+' => self.push(Token::PN(AddSubOp::ADD)),
                    b'-' => self.push(Token::PN(AddSubOp::SUB)),
                    b'?' => self.push(Token::COND),
                    b':' => self.push(Token::COLON),
                    b',' => self.push(Token::COMMA),
                    b'.' => {
                        if self.cur() == b'.' {
                            self.eat();
                            if self.cur() == b'.' {
                                self.eat();
                                self.stream.complete = false;
                                break;
                            }
                        }
                        return self.err("wrap ('...') token");
                    }
                    b'\0' => break,
                    _ => return self.err("a valid token"),
                }
            }
        }
        Ok(self.stream)
    }

    fn read_number(&mut self) -> Result<(), InvalidToken> {
        let to_digit = |c: u8| ((c as i8) - (b'0' as i8)) as i32;

        let mut num = 0.0;
        while self.cur().is_ascii_digit() {
            num *= 10.0;
            num += to_digit(self.cur()) as Real;
            self.eat()
        }

        if self.cur() == b'.' {
            self.eat();
            let mut num2 = 0.0;
            let mut div = 1.0;
            while self.cur().is_ascii_digit() {
                div *= 0.1;
                num2 += to_digit(self.cur()) as Real * div;
                self.eat()
            }
            num += num2;
        }

        if self.cur() == b'e' || self.cur() == b'E' {
            self.eat();
            let mut neg = false;
            if self.cur() == b'-' {
                neg = true;
                self.eat();
            } else if self.cur() == b'+' {
                self.eat();
            }
            if self.cur().is_ascii_digit() {
                let mut n = to_digit(self.cur());
                self.eat();
                while self.cur().is_ascii_digit() {
                    n *= 10;
                    n += to_digit(self.cur());
                    self.eat();
                }
                if neg {
                    n = -n;
                }
                num *= 10.0f64.powi(n);
            } else {
                return self.err("number index part");
            }
        }

        Ok(self.push(Token::NUM(num)))
    }

    fn cur(&self) -> u8 {
        self.line[self.column]
    }

    fn eat(&mut self) {
        self.column += 1;
    }

    fn skip_whitespace(&mut self) -> u8 {
        while self.cur().is_ascii_whitespace() {
            self.eat();
        }
        self.begin = self.column;
        return self.cur();
    }

    fn push(&mut self, token: Token) {
        self.stream.tokens.push((self.begin, token));
    }

    fn err<T>(&self, expect: &'static str) -> Result<T, InvalidToken> {
        let found = match self.cur() {
            b'\0' => "end of command".to_string(),
            c => (c as char).to_string(),
        };
        Err(InvalidToken {
            column: self.column,
            expect,
            found,
        })
    }
}
