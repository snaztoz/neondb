use crate::token::Token;

pub struct Lexer<T: Iterator<Item = char>> {
    query: T,
    ch0: Option<char>,
    ch1: Option<char>,
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
        };
        // posisikan karakter pertama di ch0
        lexer.advance();
        lexer.advance();
        lexer
    }

    pub fn lex(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = vec![];
        while self.ch0 != None {
            tokens.push(self.consume_token()?)
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
        todo!()
    }
}
