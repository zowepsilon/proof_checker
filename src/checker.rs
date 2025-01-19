use std::{collections::{HashMap, HashSet}, iter};

use crate::ast::{Formula, Proof, Statement};

#[allow(unused_imports)]
use crate::ast::ContextPrinter;

#[derive(Debug, Clone)]
struct CheckedStatement {
    hypothesis: Vec<Formula>,
    conclusion: Formula,
    parameters: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum CheckingError {
    NoInferAfterInferred(String),
    IncorrectProof,
    UnknownVariable(Formula, String),
    CannotInfer(String),
    UnificationError,
}

pub struct Checker {
    theorems: HashMap<String, CheckedStatement>,
}

impl Checker {
    pub fn new() -> Checker {
        Checker { theorems: HashMap::new() }
    }

    pub fn check(&mut self, mut stmt: Statement) -> Result<(), CheckingError> {
        let mut started_infer = false;
        let mut vars = HashSet::new();
        let mut inferred_vars = HashSet::new();
        let mut parameters = vec![];
        for (name, infer) in &stmt.args {
            vars.insert(name.clone());
            if *infer {
                inferred_vars.insert(name.clone());
                started_infer = *infer;
            } else {
                parameters.push(name.clone());
            }
            if started_infer && !infer { return Err(CheckingError::NoInferAfterInferred(name.clone())) }
        }

        for hyp in &stmt.context {
            let hv = hyp.vars();
            if !hv.is_subset(&vars) {
                let mut diff = hv.difference(&vars);

                return Err(CheckingError::UnknownVariable(hyp.clone(), diff.next().expect("hyp vars is not a subset of vars").to_string()));
            }
        }

        let prop_vars = stmt.prop.vars();
        if !prop_vars.is_subset(&vars) {
            let mut diff = prop_vars.difference(&vars);

            return Err(CheckingError::UnknownVariable(stmt.prop.clone(), diff.next().expect("hyp vars is not a subset of vars").to_string()));
        }

        if !inferred_vars.is_subset(&prop_vars) {
            let mut diff = inferred_vars.difference(&prop_vars);

            return Err(CheckingError::CannotInfer(diff.next().expect("hyp vars is not a subset of vars").to_string()));
        }

        if self.proof(&stmt.prop, &mut stmt.context, &stmt.proof) {
            self.theorems.insert(stmt.name.clone(), CheckedStatement {
                hypothesis: stmt.context,
                conclusion: stmt.prop,
                parameters,
            });

            Ok(())
        } else {
            Err(CheckingError::IncorrectProof)
        }
    }

    pub fn proof(&mut self, goal: &Formula, context: &mut Vec<Formula>, rule: &Proof) -> bool {
        //println!("Proof: {} => {}\n{}", ContextPrinter(context), goal, rule);
        match rule.name.as_str() {
            "Admitted" => true,
            "Ax" => context.iter().any(|prop| prop == goal),
            "TopIntro" => *goal == Formula::Top,
            "BottomElim" => self.proof(&Formula::Bot, context, &rule.children[0]),
            "NotIntro" => if let Formula::Not(inner) = goal {
                context.push(Formula::clone(inner));
                let res = self.proof(&Formula::Bot, context, &rule.children[0]);
                context.pop();
                res
            } else { false },
            "NotElim" => if let Formula::Bot = goal {
                self.proof(&rule.args[0], context, &rule.children[0]) 
                && self.proof(&rule.args[1], context, &rule.children[1])
            } else { false },
            "AndIntro" => if let Formula::And(left, right) = goal {
                self.proof(left, context, &rule.children[0]) && self.proof(right, context, &rule.children[1])
            } else { false },
            "AndElimLeft" => {
                self.proof(&Formula::And(Box::new(goal.clone()), Box::new(rule.args[0].clone())), context, &rule.children[0])
            },
            "AndElimRight" => {
                self.proof(&Formula::And(Box::new(rule.args[0].clone()), Box::new(goal.clone())), context, &rule.children[0])
            },
            "OrIntroLeft" => if let Formula::Or(left, _) = goal {
                self.proof(left, context, &rule.children[0])
            } else { false },
            "OrIntroRight" => if let Formula::Or(_, right) = goal {
                self.proof(right, context, &rule.children[0])
            } else { false },
            "OrElim" => {
                let hyp_left = &rule.args[0];
                let hyp_right = &rule.args[1];

                let res = self.proof(&Formula::Or(Box::new(hyp_left.clone()), Box::new(hyp_right.clone())), context, &rule.children[0]);
                
                context.push(hyp_left.clone());
                let res = res && self.proof(goal, context, &rule.children[1]);
                context.pop();
                
                context.push(hyp_right.clone());
                let res = res && self.proof(goal, context, &rule.children[2]);
                context.pop();

                res
            },
            "ImpIntro" => if let Formula::Imp(left, right) = goal {
                context.push(Formula::clone(left));
                let res = self.proof(right, context, &rule.children[0]);
                context.pop();

                res
            } else { false },
            "ImpElim" => {
                let hyp = &rule.args[0];
                self.proof(&Formula::Imp(Box::new(hyp.clone()), Box::new(goal.clone())), context, &rule.children[0])
                && self.proof(hyp, context, &rule.children[1])
            },
            name => match self.theorems.get(name) {
                Some(CheckedStatement { hypothesis, conclusion, parameters }) => {
                    use crate::unification::*;

                    let mut hypothesis = hypothesis.clone();
                    let mut conclusion = conclusion.clone();
                    
                    for hyp in &mut hypothesis {
                        add_substitution_variables(hyp, parameters);
                    }
                    add_substitution_variables(&mut conclusion, parameters);

                    if unify(&goal, &mut hypothesis, &mut conclusion).is_err() { return false; }
                    assert_eq!(assert_complete_substitution(&conclusion), Ok(()));
                    
                    let mut map = HashMap::new();
                    for (var, val) in iter::zip(parameters.iter(), rule.args.iter()) {
                        map.insert(var, val);
                    }

                    for hyp in &mut hypothesis {
                        subst_many(hyp, &map);
                    }
                    subst_many(&mut conclusion, &map);
                    
                    hypothesis.iter().enumerate().all(|(i, hyp)| self.proof(hyp, context, &rule.children[i]))
                },
                None => false,
            },
        }
    }
}

