use std::vec;

use crate::lexer::{Token, TokenData as TD};
use crate::ast::{Program, Statement, Formula, Proof};

#[derive(Debug, Clone)]
pub struct Parser {
    tokens: multipeek::MultiPeek<vec::IntoIter<Token>>,
}

macro_rules! expect {
    ($self:ident, $tok:pat) => {
        match $self.tokens.next()?.data {
            $tok => Some(()),
            other => {
                eprintln!(
                    "{}:{}:{}: expected {}, got {other:?}",
                    file!(),
                    line!(),
                    column!(),
                    stringify!($tok)
                );
                None
            }
        }
    };
}

macro_rules! list {
    ($self:ident, $to_parse:expr, sep: $sep:pat $(, end: $end:pat)?) => {
        'list: {
            $self.newlines();

            let first = match $self.tokens.peek() {
                None => break 'list Vec::new(),
                Some(Token { data, .. }) => match data {
                    $sep => {
                        let _ = $self.tokens.next();
                        $self.newlines();
                        break 'list Vec::new();
                    },
                    $($end => break 'list Vec::new(),)?
                    _ => $to_parse,
                },
            };

            let mut args = vec![first];
            $self.newlines();

            loop {
                match $self.tokens.peek() {
                    None => break,
                    Some(Token { data, .. }) => match data {
                        $sep => {
                            let _ = $self.tokens.next();
                            $self.newlines();
                        },
                        $($end => break,)?
                        _ => return None,
                    },
                };

                $self.newlines();
                args.push($to_parse);
                $self.newlines();
            }
            args
        }
    };
}

const TRACE: bool = false;

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens: multipeek::multipeek(tokens),
        }
    }

    pub fn parse(mut self) -> Option<Program> {
        if TRACE { dbg!("parse", self.tokens.peek()); }
        
        let mut program = vec![];
        loop {
            self.newlines();
            match self.tokens.peek() {
                None => break,
                Some(Token { data: TD::Prop, .. }) =>
                    match self.statement() {
                        Some(stmt) => program.push(stmt),
                        None => {
                            eprintln!("ERROR: last token {:?}", self.tokens.next());
                            return None;
                        }
                    }
                _ => {
                    eprintln!("ERROR: last token {:?}", self.tokens.next());
                    return None;
                },
            }
        }

        Some(program)
    }

    fn assert_empty(self) -> Option<()> {
        for Token { data, .. } in self.tokens {
            match data {
                TD::NewLine => (),
                _ => return None,
            }
        }

        Some(())
    }
}

impl Parser {
    fn statement(&mut self) -> Option<Statement> {
        if TRACE { dbg!("statement", self.tokens.peek()); }
        expect!(self, TD::Prop)?;
        let TD::Identifier(name) = self.tokens.next()?.data
            else { return None; };

        let TD::ParenBlock(inner) = self.tokens.next()?.data
            else { return None; };
        
        let mut inner = Parser::new(inner);
        inner.newlines();
        let args = inner.arg_list()?;
        inner.assert_empty()?;

        expect!(self, TD::Colon)?;
        expect!(self, TD::NewLine)?;

        let TD::IndentBlock(inner) = self.tokens.next()?.data
            else { return None; };

        let mut inner = Parser::new(inner);
        inner.newlines();

        let context = list!(inner,
            inner.formula()?,
            sep: TD::Comma,
            end: TD::FatArrow
        );

        expect!(inner, TD::FatArrow)?;
        let prop = inner.formula()?;

        inner.assert_empty()?;
        
        expect!(self, TD::Proof)?;
        expect!(self, TD::Colon)?;
        expect!(self, TD::NewLine)?;

        let TD::IndentBlock(inner) = self.tokens.next()?.data
            else { return None; };

        let mut inner = Parser::new(inner);
        let proof = inner.proof()?;
        inner.assert_empty()?;

        Some(Statement {
            name,
            args,
            context,
            prop,
            proof,
        })
    }

    fn arg_list(&mut self) -> Option<Vec<(String, bool)>> {
        if TRACE { dbg!("arg_list", self.tokens.peek()); }
        Some(list!(self, {
            match self.tokens.next()?.data {
                TD::Infer => {
                    if let TD::Identifier(arg) = self.tokens.next()?.data {
                        (arg, true)
                    } else {
                        return None;
                    }
                },
                TD::Identifier(arg) => (arg, false),
                _ => return None,
            }
        }, sep: TD::Comma))
    }

    fn proof(&mut self) -> Option<Proof> {
        if TRACE { dbg!("proof", self.tokens.peek()); }
        match self.tokens.next()?.data {
            TD::Identifier(name) => {
                let args = match self.tokens.next()?.data {
                    TD::Colon => vec![],
                    TD::ParenBlock(inner) => {
                        let mut inner = Parser::new(inner);

                        let args = list!(
                            inner,
                            inner.formula()?,
                            sep: TD::Comma
                        );

                        expect!(self, TD::Colon)?;

                        args
                    },
                    TD::NewLine => return Some(Proof { name, args: vec![], children: vec![] }),
                    _ => return None,
                };

                expect!(self, TD::NewLine);
                
                let TD::IndentBlock(inner) = self.tokens.next()?.data
                    else { return None; };
                let mut inner = Parser::new(inner);
                
                let mut children = vec![];
                while let Some(Token { data: TD::Identifier(_), ..}) = inner.tokens.peek() {
                    children.push(inner.proof()?);
                }

                inner.assert_empty()?;

                Some(Proof { name, args, children })
            },
            _ => None,
        }
    }

    fn newlines(&mut self) {
        if TRACE { dbg!("newlines", self.tokens.peek()); }
        while let Some(Token {
            data: TD::NewLine,
            ..
        }) = self.tokens.peek()
        {
            self.tokens.next();
        }
    }
}

macro_rules! binop {
    ($op_tok:pat, $op_tree:expr, $rule:ident, $child:ident) => {
        fn $rule(&mut self) -> Option<Formula> {
            let left = self.$child()?;

            if let Some(Token { data: $op_tok, .. }) = self.tokens.peek() {
                let _ = self.tokens.next();
                let right = self.$rule()?;
                
                Some($op_tree(Box::new(left), Box::new(right)))
            } else {
                Some(left)
            }
        }
    }
}

impl Parser {
    binop!(TD::Pipe, Formula::Or, formula, conjunction);
    binop!(TD::Dot, Formula::And, conjunction, implication);
    binop!(TD::ThinArrow, Formula::Imp, implication, primary);

    fn primary(&mut self) -> Option<Formula> {
        Some(match self.tokens.next()?.data {
            TD::Top => Formula::Top,
            TD::Bot => Formula::Bot,
            TD::Identifier(name) => Formula::Var(name),
            TD::Tilde => Formula::Not(Box::new(self.primary()?)),
            TD::ParenBlock(inner) => {
                let mut inner = Parser::new(inner);
                let result = inner.formula()?;
                inner.assert_empty();
                result
            }
            _ => return None,
        })
    }
}
