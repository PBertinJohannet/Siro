use lexer::EqLexer;
use parser::EqParser;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::mem;
use rand::random;
use std::iter::FromIterator;
use mccluskey::PrimeImplicant;
use mccluskey::mccluskey;

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

    pub fn remove_simplified(&mut self) {
        match self {
            &mut Equation::Sum(ref mut s) => s.remove_simplified(),
            _ => (),
        }
    }

    pub fn reconstruct(&mut self){
        match self {
            &mut Equation::Sum(ref mut s) => {
                let mut new_rep = mem::replace(&mut s.already_simplified, vec![]);
                s.inner.append(&mut new_rep);
                for p in s.inner.iter_mut(){
                    p.reconstruct();
                }
            },
            &mut Equation::Prod(ref mut p) => {
                for p in p.inner.iter_mut(){
                    p.reconstruct();
                }
            },
            &mut Equation::Not(ref mut n) => n.inner.reconstruct(),
            _ => ()
        }
    }

    pub fn len_removed(&mut self) -> usize{
        match self {
            &mut Equation::Sum(ref mut s) => s.already_simplified.len(),
            _ => 0,
        }
    }

    pub fn eval(&self, vars: &HashMap<String, bool>) -> bool {
        match self {
            &Equation::Sum(ref s) => s.inner.iter().any(|inner| inner.eval(&vars)),
            &Equation::Prod(ref p) => p.inner.iter().all(|inner| inner.eval(&vars)),
            &Equation::Not(ref n) => !n.inner.eval(&vars),
            &Equation::Var(ref e) => *vars.get(e)
                .unwrap_or_else(||panic!(format!("var not found : {}", e))),
        }
    }

    pub fn mccluskey(&mut self){
        let new_sum = match self {
            &mut Equation::Sum(ref mut s) => {
                let res = mccluskey(s.get_primes_implicants());
                s.inner = res;
            },
            _ => ()
        };
    }

    pub fn inners(&self) -> Vec<&Equation> {
        match self {
            &Equation::Sum(ref s) => s.inner.iter().collect(),
            &Equation::Prod(ref p) => p.inner.iter().collect(),
            &Equation::Not(ref n) => vec![&n.inner],
            v => vec![self],
        }
    }

    /// True if the truthtables are the same.
    pub fn compare_random_values(&self, other : &Equation, tests : usize) {
        let vars = other.get_vars();
        assert_eq!(vars.len(), self.get_vars().len());
        for _ in 0..tests {
            let vals = HashMap::from_iter(vars.iter().map(|&v|(v.clone(), random())));
            assert_eq!(other.eval(&vals), self.eval(&vals));
        }
    }

    pub fn into_inners(self) -> Vec<Equation> {
        match self {
            Equation::Sum(s) => s.inner,
            Equation::Prod(p) => p.inner,
            Equation::Not(n) => vec![n.inner],
            v => vec![v],
        }
    }

    pub fn complete_simplify(self) -> Self {
        let ancient_nb_var = self.get_vars().len();
        let mut old_self = self;
        let mut new_self = old_self.clone().simplified();
        while new_self != old_self {
            //println!("self len : {}", format!("{}", new_self).len());
            old_self = new_self;
            new_self = old_self.clone().simplified();
            new_self.remove_simplified();
        }
        new_self.reconstruct();
        new_self.mccluskey();
        new_self
    }

    pub fn is_product(&self) -> bool {
        match self {
            &Equation::Prod(_) => true,
            _ => false,
        }
    }

    pub fn is_simplified(&self, depth : usize) -> bool {
        match self {
            &Equation::Sum(ref s) => s
                .inner
                .iter()
                .all(|inner| inner.is_simplified(depth + 1)),
            &Equation::Prod(ref p) => p
                .inner
                .iter()
                .all(|inner| inner.is_simplified(depth + 1)),
            &Equation::Not(ref n) => n.inner.is_simplified(depth + 1),
            &Equation::Var(_) => depth < 5,
        }
    }

    /// Simplifies using simple basic rules.
    pub fn simplified(self) -> Self {
        match self {
            Equation::Sum(s) => s.simplified(),
            Equation::Prod(p) => p.simplified(),
            Equation::Not(n) => n.simplified(),
            v => v,
        }
    }

    /// Returns a list of the names of the variables.
    pub fn get_vars(&self) -> Vec<&String> {
        let mut hs = HashSet::new();
        match self {
            &Equation::Sum(ref s) => s.get_vars(),
            &Equation::Prod(ref p) => p.get_vars(),
            &Equation::Not(ref n) => n.inner.get_vars(),
            &Equation::Var(ref e) => vec![e],
        }.into_iter()
            .map(|var| hs.insert(var))
            .for_each(drop);
        let mut to_ret = hs.into_iter().collect::<Vec<&String>>();
        to_ret.sort();
        to_ret
    }

    /// Returns a list of the names of the variables.
    pub fn get_only_var(&self) -> &String {
        self.get_vars()[0]
    }


    /// Returns a list of the names of the variables.
    pub fn get_owned_vars(&self) -> Vec<String> {
        self.get_vars().iter().map(|&i|i.clone()).collect()
    }


    /// Returns the depth of the tree
    pub fn depth(&self, so_far: usize) -> usize {
        match self {
            &Equation::Sum(ref s) => s
                .inner
                .iter()
                .map(|inner| inner.depth(so_far + 1))
                .max()
                .unwrap_or(0),
            &Equation::Prod(ref p) => p
                .inner
                .iter()
                .map(|inner| inner.depth(so_far + 1))
                .max()
                .unwrap_or(0),
            &Equation::Not(ref n) => n.inner.depth(so_far + 1),
            &Equation::Var(_) => so_far + 1,
        }
    }
}

impl fmt::Display for Equation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                &Equation::Sum(ref s) => format!(
                    "({})",
                    s.inner
                        .iter()
                        .map(|inner| format!("{}", inner))
                        .collect::<Vec<String>>()
                        .join(" + ")
                ),
                &Equation::Prod(ref p) => format!(
                    "({})",
                    p.inner
                        .iter()
                        .map(|inner| format!("{}", inner))
                        .collect::<Vec<String>>()
                        .join(" * ")
                ),
                &Equation::Not(ref n) => format!("! {}", n.inner),
                &Equation::Var(ref e) => e.to_string(),
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sum {
    inner: Vec<Equation>,
    already_simplified : Vec<Equation>,
}

impl Sum {
    pub fn new(inner: Vec<Equation>) -> Self {
        Sum { inner: inner , already_simplified : vec![]}
    }

    /// Applyes the queen mccluskey algorithm to reduce the size of the sum.
    pub fn get_primes_implicants(&self) -> (Vec<String>, Vec<PrimeImplicant>){
        let vars = self.get_vars();
        (vars.clone().into_iter().map(|i|i.clone()).collect(),
         self.inner.iter().map(|ref i|PrimeImplicant::from_eq(i, &vars)).collect())
    }


    /// Returns a list of the names of the variables.
    pub fn get_owned_vars(&self) -> Vec<String> {
        self.get_vars().iter().map(|&i|i.clone()).collect()
    }

    /// Returns a list of the names of the variables.
    pub fn get_vars(&self) -> Vec<&String> {
        let mut hs = HashSet::new();
        self.inner.iter().flat_map(|inner| inner.get_vars().into_iter()).into_iter()
            .map(|var| hs.insert(var))
            .for_each(drop);
        let mut to_ret = hs.into_iter().collect::<Vec<&String>>();
        to_ret.sort();
        to_ret
    }

    /// This must be called at the top level only.
    pub fn remove_simplified(&mut self) {
        let mut not_simp = vec![];
        let mut simp = vec![];
        for i in self.inner.iter_mut(){
            if i.is_simplified(0){
                simp.push( i.clone());
            } else {
                not_simp.push(i.clone());
            }
        }
        self.inner = not_simp;
        self.already_simplified.append(&mut simp);
    }

    /// Sums can be simplified using two simple rules :
    /// sum(a) = a
    /// sum(a, sum(b, c)) = sum (a, b, c)
    pub fn simplified(mut self) -> Equation {
        let mut new_inner = vec![];
        for old_i in self.inner {
            let mut i = old_i.simplified();
            match i {
                Equation::Sum(ref mut s) => new_inner.append(&mut s.inner),
                _ => new_inner.push(i),
            };
        }
        self.inner = new_inner;
        if self.inner.len() == 1 {
            let ret = self.inner.into_iter().next().unwrap();
            return ret.simplified();
        } else {
            return Equation::Sum(Box::new(self));
        }
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

    pub fn removed_doublons(&self) -> Vec<Equation>{
        let mut inn = self.inner.clone();
        inn.sort_by_key(|a|a.get_only_var().to_string());
        let mut in_order_vars = inn.iter();
        let mut prev = match in_order_vars.next(){
            Some(p) => p,
            _ => return vec![],
        };
        let mut new_inner = vec![];
        while let Some(next) = in_order_vars.next(){
            if next.get_only_var() == prev.get_only_var(){
                // inside can only be Var or Not(Var) because it is simplified.
                if mem::discriminant(next) != mem::discriminant(prev){
                    return vec![]
                }
            } else {
                new_inner.push(prev.clone());
            }
            prev = next;
        }
        new_inner.push(prev.clone());
        new_inner.clone()
    }

    pub fn factorise_for(mut self, i: usize) -> Equation {
        let removed = self.inner.remove(i);
        let mut new_sum_of_products = vec![];
        for sub_sum_element in removed.into_inners() {
            let mut new_inner = self.inner.clone();
            new_inner.push(sub_sum_element);
            new_sum_of_products.push(Equation::Prod(Box::new(Prod::new(new_inner))));
        }
        Equation::Sum(Box::new(Sum::new(new_sum_of_products))).complete_simplify()
    }

    pub fn flatten(mut self) -> Equation {
        let mut new_inner = vec![];
        for old_i in self.inner {
            let mut i = old_i.simplified();
            match i {
                Equation::Prod(ref mut p) => new_inner.append(&mut p.inner),
                _ => new_inner.push(i),
            };
        }
        self.inner = new_inner;
        if self.inner.len() == 1 {
            let ret = self.inner.into_iter().next().unwrap();
            return ret.simplified();
        } else {
            return Equation::Prod(Box::new(self));
        }
    }
    /// Products can be simplified in the same way than the addition but we can also factorise :
    /// a * (B + c + d) * e => (a * e * B) + (a * e * c) + (a * e * d)
    ///
    pub fn simplified(mut self) -> Equation {
        self.inner = self
            .inner
            .into_iter()
            .map(|inner| inner.simplified())
            .collect();
        for i in 0..self.inner.len() {
            if mem::discriminant(&self.inner[i])
                == mem::discriminant(&Equation::Sum(Box::new(Sum::new(vec![]))))
            {
                return self.factorise_for(i);
            }
        }
        self.flatten()
    }

    /// Returns a list of the names of the variables.
    pub fn get_vars(&self) -> Vec<&String> {
        let mut hs = HashSet::new();
        self.inner.iter().flat_map(|inner| inner.get_vars().into_iter()).into_iter()
            .map(|var| hs.insert(var))
            .for_each(drop);
        let mut to_ret = hs.into_iter().collect::<Vec<&String>>();
        to_ret.sort();
        to_ret
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

    /// Returns a list of the names of the variables.
    pub fn get_only_var(&self) -> &String {
        self.inner.get_vars()[0]
    }

    pub fn simplified(mut self) -> Equation {
        self.inner = self.inner.complete_simplify();
        match self.inner {
            Equation::Not(box n) => n.inner.simplified(),
            Equation::Sum(box s) => Equation::Prod(Box::new(Prod::new(
                s.inner
                    .into_iter()
                    .map(|i| Equation::Not(Box::new(Not::new(i))))
                    .collect(),
            ))),
            Equation::Prod(box s) => Equation::Sum(Box::new(Sum::new(
                s.inner
                    .into_iter()
                    .map(|i| Equation::Not(Box::new(Not::new(i))))
                    .collect(),
            ))),
            v => Equation::Not(Box::new(Not::new(v))),
        }
    }
}

#[cfg(test)]
mod tests_eval {
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

#[cfg(test)]
mod tests_get_var {
    use super::*;
    use equation::{Equation, Not, Prod, Sum};
    use lexer::EqLexer;

    #[test]
    fn test_basics() {
        let eq = Equation::from("a + b * c".to_string());
        let mut vars = eq.get_vars();
        vars.sort();
        assert_eq!(vars, vec!["a", "b", "c"]);
    }
    #[test]
    fn test_more() {
        let eq = Equation::from("I & !B | (A + B) and (c + a./y)".to_string());
        let mut vars = eq.get_vars();
        vars.sort();
        assert_eq!(vars, vec!["A", "B", "I", "a", "c", "y"]);
    }
}

#[cfg(test)]
mod tests_depth {
    use super::*;
    use equation::{Equation, Not, Prod, Sum};
    use lexer::EqLexer;

    #[test]
    fn test_basics() {
        let eq = Equation::from("a + b * c".to_string());
        assert_eq!(eq.depth(0), 3);
    }
    #[test]
    fn test_more() {
        let eq = Equation::from("I & !B | (A + B) and (c + a./y)".to_string());
        assert_eq!(eq.depth(0), 6);
    }
}

#[cfg(test)]
mod tests_simplify {
    use super::*;
    use equation::{Equation, Not, Prod, Sum};
    use lexer::EqLexer;

    #[test]
    fn test_simplify_sum() {
        let eq = Equation::from("a + (b + c + (a + j))".to_string());
        let mut new_eq = eq.complete_simplify();
        assert_eq!(format!("{}", new_eq), "(a + b + c + j)");
    }

    #[test]
    fn test_factorise() {
        let eq = Equation::from("a * (B + c + d) * e".to_string());
        let new_eq = eq.complete_simplify();
        assert_eq!(
            format!("{}", new_eq),
            "((B * a * e) + (a * c * e) + (a * d * e))"
        );
    }

    #[test]
    fn test_flatten_prod() {
        let eq = Equation::from("a * (b * c * (a * j))".to_string());
        let new_eq = eq.simplified();
        assert_eq!(format!("{}", new_eq), "(a * b * c * a * j)");
    }

    #[test]
    fn test_remove_par() {
        let eq = Equation::from("(((a))) * ((b)) + (((c)))".to_string());
        let new_eq = eq.simplified();
        assert_eq!(format!("{}", new_eq), "((a * b) + c)");
    }


    #[test]
    fn test_not_sum() {
        let eq = Equation::from("!(a + b + c)".to_string());
        let new_eq = eq.simplified();
        assert_eq!(format!("{}", new_eq), "(! a * ! b * ! c)");
    }

    #[test]
    fn test_not_not() {
        let eq = Equation::from("!!(a + b + c)".to_string());
        let new_eq = eq.simplified().simplified();
        assert_eq!(format!("{}", new_eq), "(a + b + c)");
    }

    #[test]
    fn test_not_prod() {
        let eq = Equation::from("!(a * b * c)".to_string());
        let new_eq = eq.simplified().simplified();
        assert_eq!(format!("{}", new_eq), "(! a + ! b + ! c)");
    }

    #[test]
    fn test_keep_same() {
        let eq = Equation::from("b*  a".to_string());
        let new_eq = eq.simplified().simplified();
        assert_eq!(format!("{}", new_eq), "(b * a)");
    }

    #[test]
    fn combined_tests() {
        let eq = Equation::from(" A. B + (!((C . B) + D ) . A) + B".to_string());
        let new_eq = eq.clone().complete_simplify();
        new_eq.compare_random_values(&eq, 1000);
        let eq = Equation::from(" I & !B | (A + B) and (c + a./y)".to_string());
        let new_eq = eq.clone().complete_simplify();
        new_eq.compare_random_values(&eq, 1000);
        let eq = Equation::from("!(!a*!f + !b*!c + !d*!e)".to_string());
        let new_eq = eq.clone().complete_simplify();
        new_eq.compare_random_values(&eq, 1000);
    }

}
