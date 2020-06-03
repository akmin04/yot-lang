use crate::parser::statement::Statement;
use crate::parser::{Parser, Token};
use crate::Result;
use crate::{peek_identifier_or_err, peek_symbol_or_err};
use log::trace;

/// A yot function, either with a body or extern.
#[derive(Debug)]
pub enum Function {
    /// A regular yot function with a body.
    ///
    /// # Grammar
    /// * "@" + Identifier + "[" + (Identifier + ",")... + "]" + Statement
    RegularFunction {
        name: String,
        args: Vec<String>,
        statement: Box<Statement>,
    },

    /// An external function.
    ///
    /// # Grammar
    /// * "@!" + Identifier + "[" + (Identifier + ",")... + "]"
    ExternalFunction { name: String, args: Vec<String> },
}

impl Parser {
    pub fn parse_function(&mut self) -> Result<Function> {
        trace!("Parsing function");
        match &peek_symbol_or_err!(self)[..] {
            s @ "@" | s @ "@!" => {
                self.tokens.next();
                let name = peek_identifier_or_err!(self);
                self.tokens.next();

                if !self.next_symbol_is("[") {
                    return Err(format!("Expected `[` after function `{}`", name));
                }

                let mut args: Vec<String> = Vec::new();
                if !self.next_symbol_is("]") {
                    loop {
                        args.push(peek_identifier_or_err!(self));
                        self.tokens.next();
                        match self.tokens.next() {
                            Some(Token::Symbol(s)) if s == "]" => break,
                            Some(Token::Symbol(s)) if s == "," => (),
                            _ => {
                                return Err(format!(
                                    "Expected `]` or `,` after function `{}`",
                                    name
                                ))
                            }
                        }
                    }
                }

                if s == "@" {
                    let statement = Box::new(self.parse_statement()?);
                    Ok(Function::RegularFunction {
                        name,
                        args,
                        statement,
                    })
                } else if !self.next_symbol_is(";") {
                    Err(format!("Expected `;` after external function `{}`", name))
                } else {
                    Ok(Function::ExternalFunction { name, args })
                }
            }
            _ => Err("Expected `@` or `@!`. (Only top level functions allowed)".to_string()),
        }
    }
}
