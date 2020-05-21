mod cli;
mod generator;
mod lexer;
mod parser;

use crate::cli::OutputFormat;
use crate::generator::Generator;
use crate::lexer::tokens::Token;
use crate::lexer::Lexer;
use crate::parser::Parser;
use log::{error, warn};
use std::{fs, process};

pub fn main() {
    let cli_input = cli::init_cli();
    cli::init_logger(cli_input.verbose);

    // Lexer
    let lexer = Lexer::from_file(&cli_input.input_path).unwrap_or_else(|e| {
        error!("Lexer: {}", e);
        process::exit(1);
    });

    let tokens = lexer.collect::<Vec<_>>();
    for token in &tokens {
        if let Token::Unknown(raw) = token {
            error!("Lexer: unknown token `{}`", raw);
            process::exit(1);
        }
    }

    if cli_input.print_tokens {
        println!("***TOKENS***");
        tokens.iter().for_each(|t| println!("{:?}", t));
    }

    // Parser
    let mut parser = Parser::new(tokens.into_iter().peekable());
    let program = match parser.parse_program() {
        Ok(e) => e,
        Err(e) => {
            error!("Parser: {}", e);
            process::exit(1);
        }
    };
    if cli_input.print_ast {
        println!("***AST***\n{:#?}", program);
    }

    // Generator
    let generator = Generator::new(program, &cli_input.input_name);
    generator.generate();
    match cli_input.output_format {
        OutputFormat::LLVM => generator.generate_ir(&cli_input.output_path),
        OutputFormat::ObjectFile => {
            generator.generate_object_file(cli_input.optimization, &cli_input.output_path)
        }
        OutputFormat::Executable => {
            let object_file = format!("{}.o", cli_input.input_name);
            generator.generate_object_file(cli_input.optimization, &object_file);
            generator.generate_executable(&object_file, &cli_input.output_path);
            fs::remove_file(object_file).unwrap_or_else(|e| {
                warn!("Unable to delete object file:\n{}", e);
            });
        }
    }
}
