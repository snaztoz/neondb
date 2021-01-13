use crate::token::Token;

use std::collections::HashMap;

pub struct Lexer<T: Iterator<Item = char>> {
    query: T,
    ch0: Option<char>,
    ch1: Option<char>,
    keywords: HashMap<String, Token>,
}

fn get_keywords() -> HashMap<String, Token> {
    let mut keywords = HashMap::new();

    keywords.insert(String::from("and"), Token::And);
    keywords.insert(String::from("create"), Token::Create);
    keywords.insert(String::from("from"), Token::From);
    keywords.insert(String::from("in"), Token::In);
    keywords.insert(String::from("key"), Token::Key);
    keywords.insert(String::from("not"), Token::Not);
    keywords.insert(String::from("null"), Token::Null);
    keywords.insert(String::from("or"), Token::Or);
    keywords.insert(String::from("primary"), Token::Primary);
    keywords.insert(String::from("select"), Token::Select);
    keywords.insert(String::from("table"), Token::Table);
    keywords.insert(String::from("int"), Token::TypeInt);
    keywords.insert(String::from("char"), Token::TypeChar);

    keywords
}

impl<T> Lexer<T>
where
    T: Iterator<Item = char>,
{
    pub fn new(query: T) -> Self {
        let mut lexer = Lexer {
            query,
            ch0: None,
            ch1: None,
            keywords: get_keywords(),
        };
        // posisikan karakter pertama di ch0
        lexer.advance();
        lexer.advance();
        lexer
    }

    pub fn lex(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = vec![];
        while self.ch0 != None {
            tokens.push(self.consume_token()?);
            self.adjust_to_next_token();
        }
        Ok(tokens)
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.ch0;
        self.ch0 = self.ch1;
        self.ch1 = self.query.next();
        c
    }

    fn consume_token(&mut self) -> Result<Token, String> {
        let token = match self.ch0 {
            // Identifier adalah sebuah kata yang terdiri atas karakter
            // alphanumeric atau juga underscore, dimana karakter pertama
            // bukan merupakan karakter numerik.
            //
            // Jika karakter pertama adalah underscore, maka harus terdapat
            // karakter yang mengikutinya (tidak boleh hanya satu underscore
            // saja).
            Some('a'..='z') | Some('A'..='Z') => self.consume_identifier(),
            Some('_') => match self.ch1 {
                Some('a'..='z') | Some('A'..='Z') | Some('0'..='9') | Some('_') => {
                    self.consume_identifier()
                }

                _ => return Err(String::from("Identifier cannot be consisting of only _")),
            },

            _ => todo!(),
        };
        Ok(token)
    }

    fn consume_identifier(&mut self) -> Token {
        let mut identifier = String::new();
        loop {
            match self.ch0 {
                Some('a'..='z') | Some('A'..='Z') | Some('0'..='9') | Some('_') => {
                    identifier.push(self.advance().unwrap());
                }
                _ => break,
            }
        }

        if self.keywords.contains_key(&identifier) {
            self.keywords[&identifier].clone()
        } else {
            Token::Name(identifier)
        }
    }

    fn adjust_to_next_token(&mut self) {
        while self.ch0.is_some() && self.ch0.as_ref().unwrap().is_whitespace() {
            self.advance();
        }
    }
}
