#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Ignore,
    False,
    True,
    Not,
    LParen,
    RParen,
    And,
    Or,
    Ident(String),
}

pub struct EqLexer {
    text: Vec<char>,
    pos: usize,
}

impl EqLexer {
    pub fn new(eq: String) -> Self {
        EqLexer {
            text: eq.chars().collect(),
            pos: 0,
        }
    }

    pub fn get_tokens(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = vec![];
        while !self.is_at_end() {
            tokens.push(self.next_token()?)
        }
        Ok(tokens
            .into_iter()
            .filter(|ref t| t != &&Token::Ignore)
            .collect())
    }

    /// Peeks once for next char in the source but do not advance.
    fn peek(&mut self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.text[self.pos]
        }
    }

    /// Advance and consume a char, returning it.
    fn advance(&mut self) -> char {
        self.pos += 1;
        self.text[self.pos - 1]
    }

    pub fn is_at_end(&self) -> bool {
        self.pos >= self.text.len()
    }

    pub fn next_token(&mut self) -> Result<Token, String> {
        match self.advance() {
            '(' => Ok(Token::LParen),
            ')' => Ok(Token::RParen),
            '.' => Ok(Token::And),
            '+' => Ok(Token::Or),
            '*' => Ok(Token::And),
            '!' => Ok(Token::Not),
            '&' => Ok(Token::And),
            '|' => Ok(Token::Or),
            '/' => Ok(Token::Not),
            ' ' => Ok(Token::Ignore),
            '\r' => Ok(Token::Ignore),
            '\t' => Ok(Token::Ignore),
            '\n' => Ok(Token::Ignore),
            '1' => Ok(Token::True),
            '0' => Ok(Token::False),
            '2'...'9' => self.identifier(),
            'a'...'z' => self.identifier(),
            'A'...'Z' => self.identifier(),
            c => Err(format!("unexpected token : {}", c)),
        }
    }

    /// If it is a known keyword, register it as a keyword.
    fn identifier(&mut self) -> Result<Token, String> {
        let start = self.pos - 1;
        while self.peek().is_alphanumeric() && !self.is_at_end() {
            self.advance();
        }
        let sub_string: String = self.text[start..self.pos].into_iter().collect();
        match sub_string.as_ref() {
            "not" => Ok(Token::Not),
            "and" => Ok(Token::And),
            "or" => Ok(Token::Or),
            a => Ok(Token::Ident(a.to_string())),
        }
    }
}

#[cfg(test)]
mod tests_lexer {
    use super::*;

    #[test]
    fn test_basics() {
        assert_eq!(
            EqLexer::new("a + b".to_string()).get_tokens(),
            Ok(vec![
                Token::Ident("a".to_string()),
                Token::Or,
                Token::Ident("b".to_string()),
            ])
        );
        assert_eq!(
            EqLexer::new("I & !B | (A + B) and (c + a./y)".to_string()).get_tokens(),
            Ok(vec![
                Token::Ident("I".to_string()),
                Token::And,
                Token::Not,
                Token::Ident("B".to_string()),
                Token::Or,
                Token::LParen,
                Token::Ident("A".to_string()),
                Token::Or,
                Token::Ident("B".to_string()),
                Token::RParen,
                Token::And,
                Token::LParen,
                Token::Ident("c".to_string()),
                Token::Or,
                Token::Ident("a".to_string()),
                Token::And,
                Token::Not,
                Token::Ident("y".to_string()),
                Token::RParen,
            ])
        );
    }
}
