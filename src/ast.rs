use std::collections::HashSet;

pub type Program = Vec<Statement>;

#[derive(Debug, Clone)]
pub struct Statement {
    pub name: String,
    pub args: Vec<(String, bool)>,
    pub context: Vec<Formula>,
    pub prop: Formula,
    pub proof: Proof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Formula {
    Top,
    Bot,
    Var(String),
    Not(Box<Formula>),
    And(Box<Formula>, Box<Formula>),
    Or(Box<Formula>, Box<Formula>),
    Imp(Box<Formula>, Box<Formula>),
}

#[derive(Debug, Clone)]
pub struct Proof {
    pub name: String,
    pub args: Vec<Formula>,
    pub children: Vec<Proof>,
}

impl Formula {
    fn vars_aux(&self, acc: &mut HashSet<String>) {
        match self {
            Formula::Top => (),
            Formula::Bot => (),
            Formula::Var(v) => _ = acc.insert(v.to_string()),
            Formula::Not(f) => f.vars_aux(acc),
            Formula::And(f1, f2) | Formula::Or(f1, f2) | Formula::Imp(f1, f2) => {
                f1.vars_aux(acc);
                f2.vars_aux(acc);
            }
        }
    }
    
    pub fn vars(&self) -> HashSet<String> {
        let mut acc = HashSet::new();
        self.vars_aux(&mut acc);
        acc
    }
}

use std::fmt::Display;

impl Display for Formula {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Formula::Top => write!(f, "Top"),
            Formula::Bot => write!(f, "Bot"),
            Formula::Var(v) => write!(f, "{v}"),
            Formula::Not(inner) => write!(f, "{inner}"),
            Formula::And(left, right) => write!(f, "({left} . {right})"),
            Formula::Or(left, right) => write!(f, "({left} | {right})"),
            Formula::Imp(left, right) => write!(f, "({left} -> {right})"),
        }
    }
}

impl Display for Proof {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn inner(proof: &Proof, indent: usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{: >indent$}{}", "", proof.name)?;
            if proof.args.len() != 0 {
                write!(f, "(")?;
                let mut args = proof.args.iter().peekable();
                while let Some(arg) = args.next() {
                    if args.peek().is_none() {
                        write!(f, "{})", arg)?;
                    } else {
                        write!(f, "{}, ", arg)?;
                    }
                }
            }

            if proof.children.len() != 0 {
                write!(f, ":\n")?;
                for child in &proof.children {
                    inner(child, indent+2, f)?;
                }
            } else {
                writeln!(f)?;
            }

            Ok(())
        }

        inner(self, 0, f)
    }
}

pub struct ContextPrinter<'a>(pub &'a Vec<Formula>);

impl Display for ContextPrinter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.0.iter().peekable();
        while let Some(formula) = iter.next() {
            write!(f, "{formula}")?;
            if iter.peek().is_some() {
                write!(f, ", ")?;
            }
        }

        Ok(())
    }
}
