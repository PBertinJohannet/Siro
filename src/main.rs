#![feature(box_patterns)]
mod equation;
mod lexer;
mod parser;
use std::env;
extern crate rand;
use equation::Equation;



fn main() {
    // These are the worst
    let eq = Equation::from("!((a * b) + (b * c) + (c * d))".to_string().replace("&", ""));
    println!("eq : {}", eq);
    let new_eq = eq.complete_simplify();
    println!("depth : {}", new_eq.depth(0));
    println!("new eq len : {}", format!("{}", new_eq).len()); // 159 characters.
}
