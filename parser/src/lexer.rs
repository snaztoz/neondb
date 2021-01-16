use crate::token::Token;

use std::collections::HashMap;
use std::convert::TryInto;

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
        self.adjust_to_next_token();
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
        self.resolve_identifier(&identifier)
    }

    fn consume_number(&mut self) -> Result<Token, String> {
        let mut number = String::new();
        let mut dot_is_exist = false;
        let mut exp = 0;
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
                        exp = self.consume_exp_value()?;
                        break;
                    }
                    _ => break,
                },

                _ => break,
            }
        }
        Ok(self.resolve_number_exp(&number, exp))
    }

    fn consume_number_radix(&mut self, radix: u32) -> Result<Token, String> {
        if radix == 10 {
            return self.consume_number();
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
                    self.advance();
                    break;
                }

                Some('\\') if self.ch1 == Some(opening_quote) => {
                    self.advance();
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
                self.advance();
                Ok(Token::Asterisk)
            }

            Some(',') => {
                self.advance();
                Ok(Token::Comma)
            }

            Some('.') => {
                self.advance();
                Ok(Token::Dot)
            }

            Some('=') => {
                if self.ch1 == Some('=') {
                    self.advance();
                }
                self.advance();
                Ok(Token::Equal)
            }

            Some('>') => match self.ch1 {
                Some('=') => {
                    self.advance();
                    self.advance();
                    Ok(Token::GtEqual)
                }
                _ => {
                    self.advance();
                    Ok(Token::Gt)
                }
            },

            Some('<') => match self.ch1 {
                Some('=') => {
                    self.advance();
                    self.advance();
                    Ok(Token::LtEqual)
                }
                Some('>') => {
                    self.advance();
                    self.advance();
                    Ok(Token::NotEqual)
                }
                _ => {
                    self.advance();
                    Ok(Token::Lt)
                }
            },

            Some('!') => match self.ch1 {
                Some('=') => {
                    self.advance();
                    self.advance();
                    Ok(Token::NotEqual)
                }
                _ => {
                    self.advance();
                    Ok(Token::Exclamation)
                }
            },

            Some('(') => {
                self.advance();
                Ok(Token::ParenthL)
            }

            Some(')') => {
                self.advance();
                Ok(Token::ParenthR)
            }

            Some(';') => {
                self.advance();
                Ok(Token::Semicolon)
            }

            _ => Err(format!("unknown {} symbol", self.ch0.unwrap())),
        }
    }

    // Misal, terdapat angka 54e+03, maka angka tersebut dapat dituliskan
    // sebagai 54 * 10^3. Method ini mengambil angka '3'-nya.
    fn consume_exp_value(&mut self) -> Result<i32, String> {
        let exp_sign = self.advance().unwrap();
        if exp_sign != 'E' && exp_sign != 'e' {
            panic!("expecting first character to be 'E' or 'e'");
        }

        let mut number = String::new();

        match self.ch0 {
            Some('+') | Some('-') => match self.ch1 {
                // tidak boleh ada karakter whitespace yang memisahkan
                Some('0'..='9') => {
                    let sign = self.advance().unwrap();
                    number.push(sign);
                }

                _ => return Err(String::from("invalid scientific notation format")),
            },
            _ => return Err(String::from("invalid scientific notation format")),
        }

        loop {
            match self.ch0 {
                Some('0'..='9') => number.push(self.advance().unwrap()),
                _ => break,
            }
        }
        Ok(number.parse().unwrap())
    }

    fn resolve_identifier(&self, identifier: &str) -> Token {
        let lower = identifier.to_ascii_lowercase();
        if self.keywords.contains_key(&lower) {
            self.keywords[&lower].clone()
        } else {
            Token::Name(String::from(identifier))
        }
    }

    fn resolve_number_exp(&self, number: &str, exp: i32) -> Token {
        if number.contains('.') || exp.is_negative() {
            let number = number.parse::<f64>().unwrap() * 10f64.powi(exp);
            Token::Float(number)
        } else {
            let exp = exp.try_into().unwrap();
            let number = number.parse::<i64>().unwrap() * 10i64.pow(exp);
            Token::Int(number)
        }
    }

    // Helper untuk memastikan bilangan dengan basis selain desimal
    // tidak mengandung floating-point maupun notasi saintifik.
    fn guard_radix_number(&self, radix: u32) -> Result<(), String> {
        if radix != 2 && radix != 8 && radix != 16 {
            panic!("unknown radix encountered");
        }

        match self.ch0 {
            Some('.') => {
                let float_err = format!("floating-point is not supported in base-{} number", radix);
                match self.ch1 {
                    Some('0'..='1') if radix == 2 => Err(float_err),
                    Some('0'..='7') if radix == 8 => Err(float_err),
                    Some('0'..='9') | Some('A'..='F') | Some('a'..='f') if radix == 16 => {
                        Err(float_err)
                    }
                    _ => Ok(()),
                }
            }

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
