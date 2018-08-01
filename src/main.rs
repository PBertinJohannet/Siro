#![feature(box_patterns)]
mod equation;
mod lexer;
mod parser;
use std::env;
extern crate rand;
use equation::Equation;
mod mccluskey;


fn main() {
    // These are the worst
    let eq = Equation::from("!e * (k * !((a*b)+(c*d)))".to_string().replace("&", ""));
    println!("eq : {}", eq);
    let new_eq = eq.complete_simplify();
    println!("depth : {}", new_eq.depth(0));
    println!("new eq len : {}", format!("{}", new_eq).len()); // 159 characters.
    println!("new eq len : {}", new_eq);
}
