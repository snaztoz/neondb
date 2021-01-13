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
