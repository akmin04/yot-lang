pub mod tokens;

use crate::lexer::tokens::*;
use log::{debug, error, info};
use std::iter::Peekable;
use std::vec::IntoIter;
use std::{fs, io, process};

/// A lexical analyzer that splits the program into [`Token`]s.
///
/// [`Token`]: tokens/enum.Token.html
pub struct Lexer {
    /// The raw program characters.
    raw_data: Peekable<IntoIter<char>>,
}

impl Lexer {
    /// Creates a lexer from a program file given the path to the file.
    ///
    /// # Arguments
    /// * `file_path` - The path to the program file.
    pub fn from_file(file_path: &str) -> io::Result<Self> {
        Ok(Self::from_text(&fs::read_to_string(file_path)?))
    }

    /// Creates a lexer given the program data as plain text.
    ///
    /// # Arguments
    /// * `text` - The raw program.
    pub fn from_text(text: &str) -> Self {
        Lexer {
            raw_data: text.chars().collect::<Vec<_>>().into_iter().peekable(),
        }
    }

    /// Create a token by eating characters while a condition is met.
    ///
    /// # Arguments
    /// * `raw_token` - The raw string token to append characters to.
    /// * `cond` - The condition that must be met.
    fn get_next_char_while(&mut self, raw_token: &mut String, cond: fn(char) -> bool) {
        loop {
            match self.raw_data.peek() {
                Some(c) if cond(*c) => {
                    raw_token.push(*c);
                    self.raw_data.next();
                }
                _ => {
                    debug!(
                        "Stopping get_next_char_while after peeking {:?}",
                        self.raw_data.peek()
                    );
                    break;
                }
            }
        }
    }

    /// Check if a character is a part of an identifier.
    ///
    /// Identifiers must start with an alphabetic character or underscore, but can then include
    /// alphanumeric characters and underscores after.
    ///
    /// # Arguments
    /// * `c` - The character to check.
    fn is_identifier(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }
}

impl Iterator for Lexer {
    type Item = Token;

    /// Identifies the next token, `None` if the end of the program has been reached.
    fn next(&mut self) -> Option<Token> {
        let token: Token;
        let first_char: char;

        // Find first non-whitespace character
        loop {
            match self.raw_data.next() {
                Some(c) if c.is_whitespace() => continue,
                Some(c) => {
                    first_char = c;
                    break;
                }
                None => return None,
            }
        }

        debug!("First char: {}", first_char);

        // Identifier
        if Self::is_identifier(first_char) && !first_char.is_numeric() {
            info!("Lexing identifier");
            let mut name = first_char.to_string();
            self.get_next_char_while(&mut name, Self::is_identifier);

            token = Token::Identifier(name);
        }
        // Integer Literal
        else if first_char.is_numeric() {
            info!("Lexing integer literal");
            let mut value = first_char.to_string();
            self.get_next_char_while(&mut value, |c| c.is_numeric());

            token = match value.parse() {
                Ok(i) => Token::Literal(Literal::Integer(i)),
                Err(_) => {
                    // TODO put somehwere else
                    error!("Integer literal {} is invalid", value);
                    process::exit(1);
                }
            }
        }
        // String Literal
        else if first_char == '"' {
            info!("Lexing string literal");
            let mut value = String::new();

            self.get_next_char_while(&mut value, |c| c != '"');
            self.raw_data.next(); // Eat ending "

            token = Token::Literal(Literal::Str(value));
        }
        // Symbol or Unknown
        else {
            info!("Lexing symbol");
            let mut raw = first_char.to_string();
            loop {
                if let Some(peek) = self.raw_data.peek() {
                    raw.push(*peek);
                } else {
                    break;
                }

                if VALID_SYMBOLS.contains(&&raw[..]) {
                    self.raw_data.next();
                } else {
                    raw.pop();
                    break;
                }
            }

            token = match &raw[..] {
                // Ignore commends untill newline
                s if s == "//" => {
                    info!("Ignoring comment");
                    self.get_next_char_while(&mut String::new(), |c| c != '\n');
                    self.next()?
                }
                s if VALID_SYMBOLS.contains(&s) => Token::Symbol(raw),
                _ => Token::Unknown(raw),
            }
        }

        Some(token)
    }
}

#[cfg(test)]
mod tests {

    use super::Lexer;

    #[test]
    fn is_identifier() {
        for &i in &['a', 'z', '_', '0', '9'] {
            assert!(Lexer::is_identifier(i));
        }

        for &s in &['+', '*', '@', ';'] {
            assert!(!Lexer::is_identifier(s));
        }
    }
}
