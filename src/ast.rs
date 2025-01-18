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

#[expect(unused)]
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
