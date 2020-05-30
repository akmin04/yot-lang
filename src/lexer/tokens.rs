/// A token that is parsed by the [`Lexer`].
///
/// [`Lexer`]: ../struct.Lexer.html
#[derive(Debug, PartialEq)]
pub enum Token {
    /// An identifier of a variable or function with its name.
    Identifier(String),
    /// A [`Literal`] value.
    ///
    /// [`Literal`]: enum.Literal.html
    Literal(Literal),
    /// A known symbol.
    Symbol(String),
}

/// A literal value token, either an integer or a string.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// A literal signed 32-bit integer.
    Integer(i32),
    /// A literal string.
    Str(String),
}

/// A list of valid symbols.
///
/// If a symbol is not in this list, it will be regarded as an [`Unknown`] token and cause a lexer
/// error.
///
/// [`Unknown`]: Token::Unknown
pub const VALID_SYMBOLS: &[&str] = &[
    "=", "+", "-", "*", "/", "==", "!=", "<", ">", "<=", ">=", "?", ":", "@", "@!", "->", ";", ",",
    "{", "}", "[", "]", "(", ")", "//",
];

/// Gets the precedence of an binary operation.
///
/// Higher number meaning higher precedence. If the operation is invalid, -1 is returned.
///
/// # Arguments
/// * `op` - The binary operation.
pub fn binary_op_precedence(op: &str) -> i32 {
    match op {
        "=" => 0,
        "==" | "!=" | "<" | ">" | "<=" | ">=" => 10,
        "+" | "-" => 20,
        "*" | "/" => 30,
        _ => -1,
    }
}

/// A list of valid unary symbols.
pub const UNARY_SYMBOLS: &[&str] = &["-"];
