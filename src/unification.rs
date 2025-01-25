use std::collections::HashMap;

use crate::{ast::Formula, checker::CheckingError};

#[allow(unused_imports)]
use crate::ast::ContextPrinter;

pub fn add_substitution_variables(f: &mut Formula, except: &Vec<String>) {
    match f {
        Formula::Var(v) if !except.contains(v) => *f = Formula::Var(format!("${v}")),
        | Formula::Var(_)
        | Formula::Top
        | Formula::Bot => (),
        Formula::Not(ref mut inner) => add_substitution_variables(inner, except),
        | Formula::And(ref mut left, ref mut right)
        | Formula::Or(ref mut left, ref mut right)
        | Formula::Imp(ref mut left, ref mut right) => {
            add_substitution_variables(left, except);
            add_substitution_variables(right, except);
        },
    }
}

pub fn assert_complete_substitution(f: &Formula) -> Result<(), String> {
    match f {
        Formula::Top => Ok(()),
        Formula::Bot => Ok(()),
        Formula::Var(v) => if !v.starts_with('$') { Ok(()) } else { Err(v.clone()) },
        Formula::Not(inner) => assert_complete_substitution(inner),
        | Formula::And(left, right)
        | Formula::Or(left, right)
        | Formula::Imp(left, right) => {
            assert_complete_substitution(left)?;
            assert_complete_substitution(right)?;

            Ok(())
        }
    }
}

pub fn subst(f: &mut Formula, var: &str, value: &Formula) {
    match f {
        Formula::Var(v) if v == var => *f = value.clone(),
        | Formula::Var(_)
        | Formula::Top
        | Formula::Bot => (),
        Formula::Not(ref mut inner) => subst(inner, var, value),
        | Formula::And(ref mut left, ref mut right)
        | Formula::Or(ref mut left, ref mut right)
        | Formula::Imp(ref mut left, ref mut right) => {
            subst(left, var, value);
            subst(right, var, value);
        },
    }
}

pub fn subst_many(f: &mut Formula, map: &HashMap<&String, &Formula>) {
    match f {
        | Formula::Top
        | Formula::Bot => (),
        Formula::Var(var) => {
            match map.get(var) {
                Some(value) => *f = Formula::clone(value),
                None => ()
            }
        }
        Formula::Not(ref mut inner) => subst_many(inner, map),
        | Formula::And(ref mut left, ref mut right)
        | Formula::Or(ref mut left, ref mut right)
        | Formula::Imp(ref mut left, ref mut right) => {
            subst_many(left, map);
            subst_many(right, map);
        },
    }
}

pub fn unify(goal: &Formula, hypothesis: &mut Vec<Formula>, conclusion: &mut Formula) -> Result<(), CheckingError> {
    eprintln!("Unify: goal = {goal} <=> ccl = {conclusion} ; Hyp: {}", ContextPrinter(hypothesis));
    let mut constraints = vec![(goal.clone(), conclusion.clone())];

    while let Some((left, right)) = constraints.pop() {
        eprintln!("  {left} = {right}");
        match (left, right) {
            | (Formula::Top, Formula::Top)
            | (Formula::Bot, Formula::Bot) => (),
            | (Formula::Var(left), Formula::Var(right)) if left == right => (),
            | (Formula::Var(v), value) if v.starts_with('$') => {
                for (left, right) in &mut constraints {
                    subst(left, &v, &value);
                    subst(right, &v, &value);
                }

                for hyp in &mut *hypothesis {
                    subst(hyp, &v, &value);
                }

                subst(conclusion, &v, &value)
            },
            | (value, Formula::Var(v)) if v.starts_with('$') => {
                for (left, right) in &mut constraints {
                    subst(left, &v, &value);
                    subst(right, &v, &value);
                }

                for hyp in &mut *hypothesis {
                    subst(hyp, &v, &value);
                }

                subst(conclusion, &v, &value)
            },
            (Formula::Var(_), Formula::Var(_)) => return Err(CheckingError::UnificationError),
            (Formula::Not(inner1), Formula::Not(inner2)) => constraints.push((*inner1, *inner2)),
            | (Formula::And(left1, right1), Formula::And(left2, right2))
            | (Formula::Or(left1, right1),  Formula::Or(left2, right2))
            | (Formula::Imp(left1, right1), Formula::Imp(left2, right2)) => {
                constraints.push((*left1, *left2));
                constraints.push((*right1, *right2));
            },
            _ => return Err(CheckingError::UnificationError)
        }
    }

    eprintln!("=> {conclusion}: {}", ContextPrinter(hypothesis));
    for hyp in &mut *hypothesis {
        assert_complete_substitution(hyp).map_err(|_| CheckingError::UnificationError)?;
    }


    Ok(())
}

