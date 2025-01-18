pub type Program = Vec<Statement>;

#[expect(unused)]
#[derive(Debug, Clone)]
pub struct Statement {
    pub name: String,
    pub args: Vec<(String, bool)>,
    pub context: Vec<Formula>,
    pub prop: Formula,
    pub proof: Proof,
}

#[expect(unused)]
#[derive(Debug, Clone)]
pub enum Formula {
    Top,
    Bot,
    Var(String),
    Not(Box<Formula>),
    And(Box<Formula>, Box<Formula>),
    Or(Box<Formula>, Box<Formula>),
    Imp(Box<Formula>, Box<Formula>),
}

#[expect(unused)]
#[derive(Debug, Clone)]
pub struct Proof {
    pub name: String,
    pub args: Vec<Formula>,
    pub children: Vec<Proof>,
}
