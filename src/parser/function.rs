use crate::parser::statement::Statement;
use crate::parser::{Parser, Token};
use crate::{peek_identifier_or_err, peek_symbol_or_err};
use log::info;

/// A yot function, either with a body or extern.
///
/// Extern functions are declared with `@!` instead of `@`.
///
/// # Grammar
/// * "@" + Identifier + "[" + (Identifier + ",")... + "]" + Statement
/// * "@!" + Identifier + "[" + (Identifier + ",")... + "]"
#[derive(Debug)]
pub enum Function {
    RegularFunction {
        name: String,
        args: Vec<String>,
        statement: Box<Statement>,
    },
    ExternalFunction {
        name: String,
        args: Vec<String>,
    },
}

type MaybeFunction = Result<Function, &'static str>;

impl Parser {
    pub fn parse_function(&mut self) -> MaybeFunction {
        info!("Parsing function");
        match &peek_symbol_or_err!(self)[..] {
            s @ "@" | s @ "@!" => {
                self.tokens.next();
                let name = peek_identifier_or_err!(self);
                self.tokens.next();

                if !self.next_symbol_is("[") {
                    return Err("Expected `[` after function name");
                }

                let mut args: Vec<String> = Vec::new();
                if !self.next_symbol_is("]") {
                    loop {
                        args.push(peek_identifier_or_err!(self));
                        self.tokens.next();
                        match self.tokens.next() {
                            Some(Token::Symbol(s)) if s == "]" => break,
                            Some(Token::Symbol(s)) if s == "," => (),
                            _ => return Err("Expected `]` or `,` in function arguments"),
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
                    Err("Expected `;` after external function declaration")
                } else {
                    Ok(Function::ExternalFunction { name, args })
                }
            }
            _ => Err("Expected `@` or `@!`. (Only top level functions allowed)"),
        }
    }
}
