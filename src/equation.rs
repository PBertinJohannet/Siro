#[derive(Debug, Clone, PartialEq)]
pub enum Equation {
    Sum(Box<Sum>),
    Prod(Box<Prod>),
    Not(Box<Not>),
    Var(String),
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
