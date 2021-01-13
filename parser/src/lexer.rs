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

    fn advance(&mut self) -> Option<char> {
        let c = self.ch0;
        self.ch0 = self.ch1;
        self.ch1 = self.query.next();
        c
    }
}
