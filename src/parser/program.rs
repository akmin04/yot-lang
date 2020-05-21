use crate::parser::function::Function;
use crate::parser::Parser;
use log::{info, warn};

/// A yot program, a.k.a. the root of the abstract syntax tree.
///
/// # Grammar
/// * Function... + EOF
#[derive(Debug)]
pub struct Program {
    pub functions: Vec<Function>,
}

type MaybeProgram = Result<Program, &'static str>;

impl Parser {
    pub fn parse_program(&mut self) -> MaybeProgram {
        info!("Parsing program");
        let mut functions: Vec<Function> = Vec::new();

        loop {
            if self.tokens.peek().is_none() {
                break;
            }
            functions.push(self.parse_function()?);
        }

        let main_fn = functions.iter().any(|f| {
            if let Function::RegularFunction {
                name,
                args: _,
                statement: _,
            } = f
            {
                name == "main"
            } else {
                false
            }
        });
        if !main_fn {
            warn!("No main function found");
        }
        Ok(Program { functions })
    }
}
