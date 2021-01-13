pub enum Token {
    // tipe dasar
    Float(f64),
    Int(i64),
    Name(String),
    Str(String),

    // simbol dan operator
    Asterisk,
    Equal,
    Gt,
    GtEqual,
    Lt,
    LtEqual,
    NotEqual,
    ParenthL,
    ParenthR,

    // keyword
    And,
    Create,
    From,
    In,
    Key,
    Not,
    Null,
    Or,
    Primary,
    Select,
    Table,
    TypeInt,
    TypeChar,
}
