use crate::lexer::Lexer;
use crate::token::Token;

#[test]
fn lex_query() {
    assert!({
        let query = "SELECT * FROM my_table;";
        let mut lexer = Lexer::new(query.chars());
        let tokens = lexer.lex().unwrap();

        tokens
            == vec![
                Token::Select,
                Token::Asterisk,
                Token::From,
                Token::Name(String::from("my_table")),
                Token::Semicolon,
            ]
    });
}

#[test]
fn lex_query_with_number() {
    assert!({
        let query = "SELECT 123 563.6 0b11 0o54 0xFF;";
        let mut lexer = Lexer::new(query.chars());
        let tokens = lexer.lex().unwrap();

        tokens
            == vec![
                Token::Select,
                Token::Int(123),
                Token::Float(563.6),
                // Token::Int(123000), ?todo => bug detected
                Token::Int(3),
                Token::Int(44),
                Token::Int(255),
                Token::Semicolon,
            ]
    });
}
