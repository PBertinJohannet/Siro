#![feature(box_patterns)]
mod equation;
mod lexer;
mod parser;
use std::env;
extern crate rand;
use equation::Equation;


fn main() {
    let eq = Equation::from("a + b * c or not ! / i & b and c +(e.o)".to_string());
    let new_eq = eq.complete_simplify();
    println!("new eq is : {}", new_eq);
}
