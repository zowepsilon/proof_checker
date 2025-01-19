#[derive(Debug, Clone, PartialEq)]
pub enum TokenData {
    // structure
    ParenBlock(Vec<Token>),
    IndentBlock(Vec<Token>),

    Colon,
    Comma,
    Dot,
    ThinArrow,
    Pipe,
    FatArrow,
    Tilde,

    // literals
    Identifier(String),

    // keywords
    Prop,
    Proof,
    Infer,

    // specials
    NewLine,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position { pub line: usize, pub column: usize }

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub data: TokenData,
    pub pos: Position,
}

#[derive(Debug)]
pub struct Lexer<'a> {
    chars: multipeek::MultiPeek<std::str::Chars<'a>>,
    ok: bool,
    current: Position,
    indent_level: usize,
    after_newline: bool,
}

impl<'a> Lexer<'a> {
    fn token(&self, data: TokenData) -> Token {
        Token {
            data,
            pos: self.current,
        }
    }

    fn new_line(&mut self) {
        self.current.column = 0;
        self.current.line += 1;
    }

    fn identifier(&mut self, start: char) -> Token {
        let mut content = start.to_string();
        let start_position = self.current;

        loop { match self.chars.peek() {
            Some(ch) if ch.is_alphanumeric() || *ch == '_' => {
                self.current.column += 1;
                content.push(
                    self.chars
                        .next()
                        .expect("self.chars was peeked"),
                );
            }
            _ => {
                return Token {
                    data: match content.as_str() {
                        "Prop" => TokenData::Prop,
                        "Proof" => TokenData::Proof,
                        "infer" => TokenData::Infer,
                        _ => TokenData::Identifier(content),
                    },
                    pos: start_position,
                }
            }
        }}
    }

    pub fn new(source: &'a str) -> Self {
        Lexer {
            chars: multipeek::multipeek(source.chars()),
            current: Position {
                line: 1,
                column: 0
            },
            ok: true,
            indent_level: 0,
            after_newline: true,
        }
    }

    pub fn lex(&mut self) -> Option<Vec<Token>> {
        let mut tokens = vec![];
        for tok in self.by_ref() {
            tokens.push(tok);
        }
        
        if self.ok {
            Some(tokens)
        } else {
            eprintln!("{:?}", self.current);
            None
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        use TokenData::*;

        macro_rules! two_char_token {
            ($second_char:expr, $short_tok:expr, $long_tok:expr) => {
                match self.chars.peek() {
                    Some($second_char) => {
                        self.current.column += 1;
                        self.chars.next();
                        let token = $long_tok;

                        token
                    }
                    _ => $short_tok,
                }
            };
        }

        self.current.column += 1;
        if let Some(')') = self.chars.peek() {
            return None;
        }

        if self.after_newline {
            for i in 0..self.indent_level*2 {
                if *self.chars.peek_nth(i)? != ' ' {
                    return None;
                }
            }

            for _ in 0..self.indent_level*2 {
                let _ = self.chars.next();
            }

            if *self.chars.peek()? == ' ' {
                let _ = self.chars.next();
                let mut sub = Lexer {
                    chars: self.chars.clone(),
                    current: self.current,
                    ok: true,
                    indent_level: self.indent_level+1,
                    after_newline: false,
                };

                return match sub.lex() {
                    Some(tokens) => {
                        self.current = sub.current;
                        self.chars = sub.chars;

                        Some(self.token(IndentBlock(tokens)))
                    },
                    None => {
                        self.ok = false;
                        return None;
                    },
                }
            }
        }

        self.after_newline = false;

        let ch = self.chars.next()?;
        Some(match ch {
            '(' => {
                let mut sub = Lexer {
                    chars: self.chars.clone(),
                    current: self.current,
                    ok: true,
                    indent_level: self.indent_level,
                    after_newline: false,
                };

                match sub.lex() {
                    Some(tokens) => {
                        self.current = sub.current;
                        self.chars = sub.chars;

                        match self.chars.next() {
                            Some(')') => self.token(ParenBlock(tokens)),
                            _ => {
                                self.ok = false;
                                return None;
                            },
                        }
                    },
                    None => {
                        self.ok = false;
                        return None;
                    },
                }
            },
            ')' | '}' => unreachable!("end of block should have been detected"),
            ':' => self.token(Colon),
            ',' => self.token(Comma),
            '|' => self.token(Pipe),
            '~' => self.token(Tilde),
            '.' => self.token(Dot),
            '=' => two_char_token!(
                '>', 
                {
                    self.ok = false;
                    return None;
                },
                self.token(FatArrow)
            ),
            '-' => two_char_token!(
                '>', 
                {
                    self.ok = false;
                    return None;
                },
                self.token(ThinArrow)
            ),
            '/' => two_char_token! {
                '/',
                {
                    self.ok = false;
                    return None;
                },
                {
                    loop {
                        if matches!(self.chars.next(), Option::Some('\n') | None) {
                            self.new_line();
                            break;
                        }
                    }
                    self.next()?
                }
            },
            '\n' => {
                self.new_line();
                self.after_newline = true;

                self.token(NewLine)
            },
            letter if letter.is_alphabetic() || letter == '_' => self.identifier(letter),
            ' ' | '\t' => self.next()?,
            other => {
                dbg!(other);
                self.ok = false;
                return None;
            }
        })
    }
}
