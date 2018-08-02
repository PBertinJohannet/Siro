/// Creates
///
use equation::{Equation, Sum, Prod, Not};
use std::collections::HashSet;
#[derive(Debug, PartialOrd, PartialEq, Copy, Clone, Hash, Eq)]
pub enum Present {
    No,
    Yes,
    Any,
}

#[derive(Debug, Clone, PartialOrd, Hash, Eq, PartialEq)]
pub struct PrimeImplicant {
    list : Vec<Present>,
    nb_any : usize,
    nb_yes : usize,
}

impl PrimeImplicant {
    pub fn from_eq(eq : &Equation, vars : &Vec<&String>) -> Self {
        let len = vars.len();
        match eq {
            &Equation::Sum(ref s) => panic!("do not give a sum to from_eq"),
            &Equation::Not(ref n) => Self::any_with(vars.iter().position(|&x| x == n.get_only_var()).unwrap(), Present::No, len),
            &Equation::Prod(ref p) => Self::from_prod(p, vars),
            &Equation::Var(ref v) => Self::any_with(vars.iter().position(|&x| x == v).unwrap(), Present::Yes, len),
        }
    }
    pub fn from_prod(p : &Prod, vars : &Vec<&String>) -> Self {
        let list = vars.iter().map( | &var| match p.removed_doublons().iter()
            .find( | i|i.get_only_var() == var){
                        None => Present::Any,
                        Some(Equation::Not(_)) => Present::No,
                        Some(Equation::Var(_)) => Present::Yes,
                        _ => panic ! ("cant have sums or non flattened products in from_prod"),
                }).collect::<Vec<Present>>();
        let nb_any= list.iter().filter(|&p|p == &Present::Any).count();
        let nb_yes= list.iter().filter(|&p|p == &Present::Yes).count();
        PrimeImplicant {
            list: list,
            nb_any: nb_any,
            nb_yes: nb_yes,
        }
    }
    pub fn any_with(pos : usize, val : Present, len : usize) -> Self {
        PrimeImplicant {
            list: (0..len).map(|i|match i == pos {
                true => val,
                _ => Present::Any
            }).collect(),
            nb_any: len-1,
            nb_yes: match val {
                Present::Yes => 1,
                _ => 0,
            },
        }
    }

    pub fn to_eq(self, vars : &Vec<&String>) -> Equation {
        Equation::Prod(Box::new(Prod::new(self.list.into_iter().zip(vars.iter()).filter_map(|(pres, var)| match pres {
            Present::Yes => Some(Equation::Var(var.to_string())),
            Present::No => Some(Equation::Not(Box::new(Not::new(Equation::Var(var.to_string()))))),
            Present::Any => None
        }).collect()))).simplified()
    }

    /// Checks that there is only one difference
    pub fn can_merge(&self, other : &Self) -> bool {
        self.list.iter().zip(other.list.iter()).filter(|(a, b)|a!=b).count() == 1
    }

    /// Checks that there is only one difference
    pub fn merge(&self, other : &Self) -> PrimeImplicant {
        let list = self.list.iter().zip(other.list.iter()).map(|(&a, &b)|if a!=b {Present::Any} else {a}).collect::<Vec<Present>>();
        let nb_any= list.iter().filter(|&p|p == &Present::Any).count();
        let nb_yes= list.iter().filter(|&p|p == &Present::Yes).count();
        PrimeImplicant {
            list: list,
            nb_any: nb_any,
            nb_yes: nb_yes,
        }
    }

    /// returns a string in the form 01001xx0 representing the inner list
    pub fn get_string(&self) -> String {
        self.list.iter().map(|v|match v {
            Present::Any => 'x',
            Present::No => '0',
            Present::Yes => '1',
        }).collect()
    }
}

/// Returns an array of arrays of primes implicants sorted by nb_any
/// the index in the first array is the number of yes in the sub array.
pub fn sort_prime_implicants(mut impls : Vec<PrimeImplicant>) -> Vec<Vec<PrimeImplicant>>{
    impls.sort_by_key(|i|i.nb_yes);
    let mut impls_by_yes = vec![];
    let mut current_impls = vec![];
    let mut current_yes = 1;
    for i in impls {
        if i.nb_yes > current_yes {
            while i.nb_yes > current_yes+1 {
                current_yes+=1;
                impls_by_yes.push(vec![]);
            }
            impls_by_yes.push(current_impls);
            current_impls = vec![i];
        } else {
            current_impls.push(i);
        }
    }
    impls_by_yes.push(current_impls);
    impls_by_yes
}
/// Take a primeimplicant (eg 0100x1) and a list of primes implicants that have 1 more yes and are
/// sorted by number of any and compare them.
/// If merging is needed, remove the prime implicant from the others.
pub fn merge_similar(prime : PrimeImplicant, others : &mut Vec<PrimeImplicant>)->Option<PrimeImplicant>{
    for i in 0..others.len() {
        if prime.can_merge(&others[i]){
            let to_ret = prime.merge(&others[i]);
            others.remove(i);
            return Some(to_ret)
        }
    }
    None
}

pub fn mccluskey((vars, mut impls) : (Vec<String>, Vec<PrimeImplicant>)) -> Vec<Equation>{
    let mut hs = HashSet::new();
    mccluskey_primes(impls)
    .into_iter()
    .map(|var| hs.insert(var))
    .for_each(drop);
    let mut to_ret = hs.into_iter().collect::<Vec<PrimeImplicant>>();
    to_ret.sort_by_key(|p|p.get_string());
    to_ret.into_iter().map(|p|p.to_eq(&vars.iter().collect())).collect()
}

/// Checks for merge on implicants differing by one Yes
pub fn mccluskey_pass_one(mut sorted_primes : Vec<Vec<PrimeImplicant>>) -> Vec<PrimeImplicant> {
    let mut to_add = vec![];
    for nb_yes in 0..sorted_primes.len()-1 { // for every list
        let (arr_now , arr_next) = sorted_primes.split_at_mut(nb_yes+1);
        let (now, next) = (arr_now.last_mut().unwrap(), arr_next.first_mut().unwrap());
        for base_id in 0..now.len(){
            match merge_similar(now[base_id].clone(), next){
                Some(new) => to_add.push(new),
                None => to_add.push(now[base_id].clone()),
            }
        }
    }
    to_add.append(&mut sorted_primes.last_mut().unwrap());
    to_add
}
/// Merge on implicants differing by one Any
pub fn mccluskey_pass_two(mut sorted_primes : Vec<Vec<PrimeImplicant>>) -> Vec<PrimeImplicant> {
    let mut to_add = vec![];
    while let Some(mut now) = sorted_primes.pop(){
        while let Some(mut curr) = now.pop(){
            match merge_similar(curr.clone(), &mut now){
                Some(new) => to_add.push(new),
                None => to_add.push(curr),
            }
        }
    }
    to_add
}

pub fn mccluskey_primes(mut impls : Vec<PrimeImplicant>)-> Vec<PrimeImplicant>{
    let mut size_before = impls.len();
    let mut sorted_primes = sort_prime_implicants(impls); // l of l
    loop {
        let mut to_add = mccluskey_pass_two(sort_prime_implicants(mccluskey_pass_one(sorted_primes)));
        if size_before == to_add.len(){
            return to_add;
        }
        size_before = to_add.len();
        sorted_primes = sort_prime_implicants(to_add);
    }
}



#[cfg(test)]
mod tests_mccluskey_primes {
    use super::*;
    use equation::{Equation, Not, Prod, Sum};

    fn prime_from_prod(eq: &'static str) -> PrimeImplicant {
        let v = vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string(), "e".to_string()];
        let vars = v.iter().collect();
        PrimeImplicant::from_eq(&Equation::from(eq.to_string()).complete_simplify(), &vars)
    }

    fn simplified_sum(vars : Vec<String>, prods : Vec<PrimeImplicant>) -> String {
        format!("{}", Equation::Sum(Box::new(Sum::new(mccluskey((vars, prods))))))
    }

    #[test]
    fn test_yes_and_not() {
        let vars = vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string(), "e".to_string()];
        assert_eq!(simplified_sum(vars,vec![prime_from_prod("a*b*!c"),
                     prime_from_prod("a*!b*!c"),
                     prime_from_prod("b*c")]), "((a * ! c) + (b * c))");
    }

    #[test]
    fn test_just_one() {
        let vars = vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string(), "e".to_string()];
        assert_eq!(simplified_sum(vars.clone(),vec![prime_from_prod("a*b*!c"),
                     prime_from_prod("a*!c"),
                     prime_from_prod("b*c")]), "((a * ! c) + (b * c))");
        assert_eq!(simplified_sum(vars,vec![prime_from_prod("a*!b*!c"),
                     prime_from_prod("a*!c"),
                     prime_from_prod("b*c")]), "((a * ! c) + (b * c))");
    }

    #[test]
    fn test_complete() {
        let vars = vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string(), "e".to_string()];
        assert_eq!(simplified_sum(vars,vec![prime_from_prod("a*!b*!c"),
                     prime_from_prod("a*!c"),
                     prime_from_prod("b*a*c"),
                     prime_from_prod("!b*c"),
                     prime_from_prod("b*c")]), "((a * ! c) + c)");
    }
}



#[cfg(test)]
mod tests_merging {
    use super::*;
    use equation::{Equation, Not, Prod, Sum};

    fn prime_from_prod(eq : &'static str) -> PrimeImplicant {
        let v = vec!["c".to_string(), "b".to_string(), "e".to_string(), "d".to_string(), "a".to_string()];
        let vars = v.iter().collect();
        PrimeImplicant::from_eq(&Equation::from(eq.to_string()).complete_simplify(), &vars)
    }

    #[test]
    fn test_can_merge() {
        assert_eq!(prime_from_prod("!a*b*d").can_merge(&prime_from_prod("!a*b")), true);
        assert_eq!(prime_from_prod("!a*b*d").can_merge(&prime_from_prod("a*b*d")), true);
        assert_eq!(prime_from_prod("!a*b*d").can_merge(&prime_from_prod("a*b*!d")), false);
        assert_eq!(prime_from_prod("!a*c").can_merge(&prime_from_prod("a")), false);
        assert_eq!(prime_from_prod("!a*c").can_merge(&prime_from_prod("!a")), true);
        assert_eq!(prime_from_prod("!a").can_merge(&prime_from_prod("a")), true);
        assert_eq!(prime_from_prod("a*b*c*d*e").can_merge(&prime_from_prod("a*e*b*c")), true);
        assert_eq!(prime_from_prod("!a*b*c*d*e").can_merge(&prime_from_prod("a*e*b*c")), false);
        assert_eq!(prime_from_prod("a*b").can_merge(&prime_from_prod("a*e")), false);
    }

    #[test]
    fn test_merge() {
        let v = vec!["c".to_string(), "b".to_string(), "e".to_string(), "d".to_string(), "a".to_string()];
        let vars = v.iter().collect();
        assert_eq!(format!("{}", prime_from_prod("!a*b*d").merge(&prime_from_prod("!a*b")).to_eq(&vars)),
                   "(b * ! a)".to_string());
        assert_eq!(format!("{}", prime_from_prod("!a*b*d").merge(&prime_from_prod("a*b*d")).to_eq(&vars)),
                   "(b * d)".to_string());
        assert_eq!(format!("{}", prime_from_prod("!a*c").merge(&prime_from_prod("!a")).to_eq(&vars)),
                   "! a".to_string());
        assert_eq!(format!("{}", prime_from_prod("!a").merge(&prime_from_prod("a")).to_eq(&vars)),
                   "()".to_string());
        assert_eq!(format!("{}", prime_from_prod("a*b*c*d*e").merge(&prime_from_prod("a*e*b*c")).to_eq(&vars)),
                   "(c * b * e * a)".to_string());
    }
}
#[cfg(test)]
mod tests_to_eq {
    use super::*;
    use equation::{Equation, Not, Prod, Sum};

    #[test]
    fn test_complete() {
        let v = vec!["c".to_string(), "b".to_string(), "e".to_string(), "d".to_string(), "a".to_string()];
        let vars = v.iter().collect();
        assert_eq!(Equation::from("c*!b*e*a".to_string()).complete_simplify(),
                   PrimeImplicant {
                        list : vec![Present::Yes,Present::No,Present::Yes,Present::Any, Present::Yes],
                        nb_any : 1,
                        nb_yes : 3,
                    }.to_eq(&vars))
    }

}
#[cfg(test)]
mod tests_from_eq {
    use super::*;
    use equation::{Equation, Not, Prod, Sum};

    #[test]
    fn test_from_not() {
        let v = vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()];
        let vars = v.iter().collect();
        assert_eq!(PrimeImplicant::from_eq(&Equation::from("!a".to_string()).complete_simplify(), &vars),
                   PrimeImplicant {
                        list : vec![Present::No,Present::Any,Present::Any,Present::Any],
                        nb_any : 3,
                        nb_yes : 0,
                    })
    }
    #[test]
    fn test_from_prod() {
        let v = vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()];
        let vars = v.iter().collect();
        assert_eq!(PrimeImplicant::from_eq(&Equation::from("b*a".to_string()).complete_simplify(), &vars),
                   PrimeImplicant {
                        list : vec![Present::Yes,Present::Yes,Present::Any,Present::Any],
                        nb_any : 2,
                        nb_yes : 2,
                    })
    }
    #[test]
    fn test_from_var() {
        let v = vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()];
        let vars = v.iter().collect();
        assert_eq!(PrimeImplicant::from_eq(&Equation::from("c".to_string()).complete_simplify(), &vars),
                   PrimeImplicant {
                        list : vec![Present::Any,Present::Any,Present::Yes,Present::Any],
                        nb_any : 3,
                        nb_yes : 1,
                    })
    }
    #[test]
    fn test_from_multi() {
        let v = vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string(), "e".to_string()];
        let vars = v.iter().collect();
        assert_eq!(PrimeImplicant::from_eq(&Equation::from("c*!b*e*e*a".to_string()).complete_simplify(), &vars),
                   PrimeImplicant {
                        list : vec![Present::Yes,Present::No,Present::Yes,Present::Any, Present::Yes],
                        nb_any : 1,
                        nb_yes : 3,
                    })
    }
}