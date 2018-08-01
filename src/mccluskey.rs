/// Creates
///
use equation::{Equation, Sum, Prod, Not};
#[derive(Debug, PartialOrd, PartialEq, Copy, Clone)]
pub enum Present {
    No,
    Yes,
    Any,
}

#[derive(Debug, PartialEq, Clone)]
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
        println!("for p: {:?}", p);
        println!("tried on : {:?}", p.removed_doublons());
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
        }).collect())))
    }
}

/// Returns an array of arrays of primes implicants sorted by nb_any
/// the index in the first array is the number of yes in the sub array.
pub fn sort_prime_implicants(mut impls : Vec<PrimeImplicant>) -> Vec<Vec<PrimeImplicant>>{
    impls.sort_by_key(|i|i.nb_yes);
    let mut impls_by_yes = vec![];
    let mut current_impls = vec![];
    let mut current_yes = 0;
    for i in impls {
        if i.nb_yes > current_yes {
            current_yes+=1;
            impls_by_yes.push(current_impls);
            current_impls = vec![i];
        } else {
            current_impls.push(i);
        }
    }
    impls_by_yes
}
/// Take a primeimplicant (eg 0100x1) and a list of primes implicants that have 1 more yes and are
/// sorted by number of any and compare them.
/// If merging is needed, remove the prime implicant from the others.
pub fn merge_similar(prime : PrimeImplicant, others : &mut Vec<PrimeImplicant>)->Option<PrimeImplicant>{
    for i in 0..others.len() {
        if prime.can_merge(others[i]){
            return Some(prime.merge(i))
        }
    }
    None
}

pub fn mccluskey((vars, mut impls) : (Vec<String>, Vec<PrimeImplicant>)) -> Vec<Equation>{
    let mut size_before = impls.len();
    let mut sorted_primes = sort_prime_implicants(impls); // l of l
    loop {
        let mut to_add = vec![];
        for nb_yes in 0..sorted_primes.len()-1 { // for every list
            let (now, next) = sorted_primes.split_at_mut(nb_yes+1);
            for base_id in 0..now.len(){
                match merge_similar(now[base_id].clone(), next){
                    Some(new) => to_add.push(new),
                    None => to_add.push(now[base_id]),
                }
            }
        }
        if size_before == to_add.len(){
            break;
        }
        size_before = to_add.len();
        sorted_primes = sort_prime_implicants(impls);
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