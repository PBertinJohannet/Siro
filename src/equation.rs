use lexer::EqLexer;
use parser::EqParser;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Equation {
    Sum(Box<Sum>),
    Prod(Box<Prod>),
    Not(Box<Not>),
    Var(String),
}

impl Equation {
    pub fn from(text: String) -> Self {
        EqParser::new(EqLexer::new(text).get_tokens().unwrap()).parse()
    }

    pub fn eval(&self, vars: &HashMap<String, bool>) -> bool {
        match self {
            &Equation::Sum(ref s) => s.inner.iter().any(|inner| inner.eval(&vars)),
            &Equation::Prod(ref p) => p.inner.iter().all(|inner| inner.eval(&vars)),
            &Equation::Not(ref n) => !n.inner.eval(&vars),
            &Equation::Var(ref e) => *vars.get(e).unwrap(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sum {
    inner: Vec<Equation>,
}
impl Sum {
    pub fn new(inner: Vec<Equation>) -> Self {
        Sum { inner: inner }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Prod {
    inner: Vec<Equation>,
}
impl Prod {
    pub fn new(inner: Vec<Equation>) -> Self {
        Prod { inner: inner }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Not {
    inner: Equation,
}
impl Not {
    pub fn new(inner: Equation) -> Self {
        Not { inner: inner }
    }
}

#[cfg(test)]
mod tests_parser {
    use super::*;
    use equation::{Equation, Not, Prod, Sum};
    use lexer::EqLexer;

    #[test]
    fn test_basics() {
        let eq = Equation::from("a + b * c".to_string());
        let mut vars = HashMap::new();
        vars.insert("a".to_string(), false);
        vars.insert("b".to_string(), false);
        vars.insert("c".to_string(), true);
        assert_eq!(eq.eval(&vars), false);
        // with a to true it is always true
        vars.insert("a".to_string(), true);
        vars.insert("b".to_string(), false);
        vars.insert("c".to_string(), true);
        assert_eq!(eq.eval(&vars), true);
        // with a to false and both true it is true
        vars.insert("a".to_string(), false);
        vars.insert("b".to_string(), true);
        vars.insert("c".to_string(), true);
        assert_eq!(eq.eval(&vars), true);
    }
    #[test]
    fn test_more() {
        let eq = Equation::from("I & !B | (A + B) and (c + a./y)".to_string());
        let mut vars = HashMap::new();
        // in this case it is true
        vars.insert("I".to_string(), true);
        vars.insert("B".to_string(), false);
        vars.insert("A".to_string(), true);
        vars.insert("c".to_string(), true);
        vars.insert("a".to_string(), false);
        vars.insert("y".to_string(), true);
        assert_eq!(eq.eval(&vars), true);
        // in this case it is false because the y is true
        vars.insert("I".to_string(), true);
        vars.insert("B".to_string(), true);
        vars.insert("A".to_string(), false);
        vars.insert("c".to_string(), false);
        vars.insert("a".to_string(), true);
        vars.insert("y".to_string(), true);
        assert_eq!(eq.eval(&vars), false);
        // setting it to false make it true
        vars.insert("I".to_string(), true);
        vars.insert("B".to_string(), true);
        vars.insert("A".to_string(), false);
        vars.insert("c".to_string(), false);
        vars.insert("a".to_string(), true);
        vars.insert("y".to_string(), false);
        assert_eq!(eq.eval(&vars), true);
    }
}
