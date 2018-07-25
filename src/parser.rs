use equation::{Equation, Not, Prod, Sum};
use lexer::Token;

struct EqParser {
    tokens: Vec<Token>,
    pos: usize,
}

impl EqParser {
    pub fn new(tokens: Vec<Token>) -> Self {
        EqParser {
            tokens: tokens,
            pos: 0,
        }
    }

    /// Peeks once for next char in the source but do not advance.
    fn peek(&mut self) -> Option<&Token> {
        if self.is_at_end() {
            None
        } else {
            Some(&self.tokens[self.pos])
        }
    }

    /// Checks that the next token is of the given type.
    pub fn check(&mut self, token: &Token) -> bool {
        !self.is_at_end() && self.peek() == Some(token)
    }

    /// Advance and consume a char, returning it.
    fn advance(&mut self) -> &Token {
        self.pos += 1;
        &self.tokens[self.pos - 1]
    }

    pub fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    pub fn parse(&mut self) -> Equation {
        self.sum()
    }

    pub fn sum(&mut self) -> Equation {
        let mut sm = vec![self.prod()];
        while self.check(&Token::Or) {
            self.advance();
            sm.push(self.prod());
        }
        Equation::Sum(Box::new(Sum::new(sm)))
    }

    pub fn prod(&mut self) -> Equation {
        let mut sm = vec![self.not()];
        while self.check(&Token::And) {
            self.advance();
            sm.push(self.not());
        }
        Equation::Prod(Box::new(Prod::new(sm)))
    }

    pub fn not(&mut self) -> Equation {
        match self.check(&Token::Not) {
            true => {
                self.advance();
                Equation::Not(Box::new(Not::new(self.literal())))
            }
            false => self.literal(),
        }
    }

    pub fn literal(&mut self) -> Equation {
        let pos = self.pos;
        match self.check(&Token::LParen) {
            true => {
                self.advance();
                let inner = self.sum();
                self.advance();
                inner
            }
            false => match self.advance() {
                Token::Ident(s) => Equation::Var(s.to_string()),
                u => panic!(format!(
                    "error, expected variable or rparen, found {:?} at : {}",
                    u, pos
                )),
            },
        }
    }
}

#[cfg(test)]
mod tests_parser {
    use super::*;
    use equation::{Equation, Not, Prod, Sum};
    use lexer::EqLexer;
    #[test]
    fn test_basics() {
        assert_eq!(
            EqParser::new(EqLexer::new("a + b".to_string()).get_tokens().unwrap()).parse(),
            Equation::Sum(Box::new(Sum::new(vec![
                Equation::Prod(Box::new(Prod::new(vec![Equation::Var("a".to_string())]))),
                Equation::Prod(Box::new(Prod::new(vec![Equation::Var("b".to_string())]))),
            ])))
        );
        assert_eq!(
            EqParser::new(
                EqLexer::new("I & !B | (A + B) and (c + a./y)".to_string())
                    .get_tokens()
                    .unwrap()
            ).parse(),
            Equation::Sum(Box::new(Sum::new(vec![
                Equation::Prod(Box::new(Prod::new(vec![
                    Equation::Var("I".to_string()),
                    Equation::Not(Box::new(Not::new(Equation::Var("B".to_string())))),
                ]))),
                Equation::Prod(Box::new(Prod::new(vec![
                    Equation::Sum(Box::new(Sum::new(vec![
                        Equation::Prod(Box::new(Prod::new(vec![Equation::Var("A".to_string())]))),
                       Equation::Prod(Box::new(Prod::new(vec![Equation::Var("B".to_string())]))),
                    ]))),
                    Equation::Sum(Box::new(Sum::new(vec![
                        Equation::Prod(Box::new(Prod::new(vec![Equation::Var("c".to_string())]))),
                        Equation::Prod(Box::new(Prod::new(vec![
                            Equation::Var("a".to_string()),
                            Equation::Not(Box::new(Not::new(Equation::Var("y".to_string())))),
                        ]))),
                    ]))),
                ]))),
            ],)))
        );
    }
}