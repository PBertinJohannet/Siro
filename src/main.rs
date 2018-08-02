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
    let eq = Equation::from("!(a+!b * !(x*z + !(!a*!f + !b*!c + !d*!e)))".to_string().replace("&", ""));
    println!("eq : {}", eq);
    let new_eq = eq.complete_simplify();
    println!("depth : {}", new_eq.depth(0));
    println!("new eq len : {}", format!("{}", new_eq).len()); // 280 characters.
    println!("new eq len : {}", new_eq);
}
