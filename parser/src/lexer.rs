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
    keywords.insert(String::from("where"), Token::Where);

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

            // Numerik.
            //
            // Untuk nilai float serta scientific notation hanya disupport
            // oleh bilangan basis 10 (desimal).
            Some('0') => match self.ch1 {
                Some('b') => {
                    self.advance();
                    self.advance();
                    self.consume_number_radix(2)?
                }
                Some('o') => {
                    self.advance();
                    self.advance();
                    self.consume_number_radix(8)?
                }
                Some('x') => {
                    self.advance();
                    self.advance();
                    self.consume_number_radix(16)?
                }
                _ => self.consume_number()?,
            },
            Some('1'..='9') => self.consume_number()?,

            // String
            Some('\'') | Some('"') => self.consume_str()?,

            _ => self.consume_symbols()?,
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

        let lower = identifier.to_ascii_lowercase();
        if self.keywords.contains_key(&lower) {
            self.keywords[&lower].clone()
        } else {
            Token::Name(identifier)
        }
    }

    fn consume_number(&mut self) -> Result<Token, String> {
        let mut number = String::new();
        let mut dot_is_exist = false;
        let mut exp_is_exist = false;
        loop {
            match self.ch0 {
                Some('0'..='9') => number.push(self.advance().unwrap()),

                Some('.') => match self.ch1 {
                    Some('0'..='9') => {
                        if dot_is_exist {
                            break;
                        }
                        dot_is_exist = true;
                        number.push(self.advance().unwrap());
                    }
                    _ => break,
                },

                Some('E') | Some('e') => match self.ch1 {
                    Some('+') | Some('-') => {
                        if exp_is_exist {
                            break;
                        }
                        exp_is_exist = true;
                        number.push(self.advance().unwrap());
                        number.push(self.advance().unwrap());
                    }
                    _ => break,
                },

                _ => break,
            }
        }
        if dot_is_exist {
            Ok(Token::Float(number.parse().unwrap()))
        } else {
            Ok(Token::Int(number.parse().unwrap()))
        }
    }

    fn consume_number_radix(&mut self, radix: u32) -> Result<Token, String> {
        if radix == 10 {
            return self.consume_number();
        } else if radix != 2 && radix != 8 && radix != 16 {
            panic!("unknown radix encountered");
        }

        let mut number = String::new();
        loop {
            self.guard_radix_number(radix)?;
            match radix {
                2 => match self.ch0 {
                    Some('0'..='1') => number.push(self.advance().unwrap()),
                    _ => break,
                },

                8 => match self.ch0 {
                    Some('0'..='7') => number.push(self.advance().unwrap()),
                    _ => break,
                },

                16 => match self.ch0 {
                    Some('0'..='9') | Some('A'..='F') | Some('a'..='f') => {
                        number.push(self.advance().unwrap())
                    }
                    _ => break,
                },

                _ => unreachable!(),
            }
        }
        Ok(Token::Int(i64::from_str_radix(&number, radix).unwrap()))
    }

    fn consume_str(&mut self) -> Result<Token, String> {
        debug_assert!(self.ch0 == Some('\'') || self.ch0 == Some('"'));

        let mut s = String::new();
        let opening_quote = self.advance().unwrap();
        loop {
            match self.ch0 {
                Some(quote) if quote == opening_quote => {
                    self.advance().unwrap();
                    break;
                }

                Some('\\') if self.ch1 == Some(opening_quote) => {
                    self.advance().unwrap();
                    s.push(self.advance().unwrap());
                }

                None => return Err(String::from("missing closing quote")),

                _ => s.push(self.advance().unwrap()),
            }
        }
        Ok(Token::Str(s))
    }

    fn consume_symbols(&mut self) -> Result<Token, String> {
        match self.ch0 {
            Some('*') => {
                self.advance().unwrap();
                Ok(Token::Asterisk)
            }

            Some(',') => {
                self.advance().unwrap();
                Ok(Token::Comma)
            }

            Some('.') => {
                self.advance().unwrap();
                Ok(Token::Dot)
            }

            Some('=') => {
                if self.ch1 == Some('=') {
                    self.advance().unwrap();
                }
                self.advance().unwrap();
                Ok(Token::Equal)
            }

            Some('>') => match self.ch1 {
                Some('=') => {
                    self.advance().unwrap();
                    self.advance().unwrap();
                    Ok(Token::GtEqual)
                }
                _ => {
                    self.advance().unwrap();
                    Ok(Token::Gt)
                }
            },

            Some('<') => match self.ch1 {
                Some('=') => {
                    self.advance().unwrap();
                    self.advance().unwrap();
                    Ok(Token::LtEqual)
                }
                Some('>') => {
                    self.advance().unwrap();
                    self.advance().unwrap();
                    Ok(Token::NotEqual)
                }
                _ => {
                    self.advance().unwrap();
                    Ok(Token::Lt)
                }
            },

            Some('!') => match self.ch1 {
                Some('=') => {
                    self.advance().unwrap();
                    self.advance().unwrap();
                    Ok(Token::NotEqual)
                }
                _ => {
                    self.advance().unwrap();
                    Ok(Token::Exclamation)
                }
            },

            Some('(') => {
                self.advance().unwrap();
                Ok(Token::ParenthL)
            }

            Some(')') => {
                self.advance().unwrap();
                Ok(Token::ParenthR)
            }

            Some(';') => {
                self.advance().unwrap();
                Ok(Token::Semicolon)
            }

            _ => Err(format!("unknown {} symbol", self.ch0.unwrap())),
        }
    }

    // Helper untuk memastikan bilangan dengan basis selain desimal
    // tidak mengandung floating-point maupun notasi saintifik.
    fn guard_radix_number(&self, radix: u32) -> Result<(), String> {
        debug_assert!(radix == 2 || radix == 8 || radix == 16);

        let err_msg = format!("floating-point is not supported in base-{} number", radix);
        match self.ch0 {
            Some('.') => match self.ch1 {
                Some('0'..='1') if radix == 2 => Err(err_msg),
                Some('0'..='7') if radix == 8 => Err(err_msg),
                Some('0'..='9') | Some('A'..='F') | Some('a'..='f') if radix == 16 => Err(err_msg),
                _ => Ok(()),
            },

            Some('E') | Some('e') => match self.ch1 {
                Some('+') | Some('-') => Err(format!(
                    "scientific notation is not supported in base-{} number",
                    radix
                )),
                _ => Ok(()),
            },

            _ => Ok(()),
        }
    }

    fn adjust_to_next_token(&mut self) {
        while self.ch0.is_some() && self.ch0.as_ref().unwrap().is_whitespace() {
            self.advance();
        }
    }
}
