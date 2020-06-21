use log::{error, warn};
use std::{fs, process};
use yotc::generator::Generator;
use yotc::lexer::Lexer;
use yotc::parser::Parser;
use yotc::{init_cli, init_logger, OutputFormat};

/// Unwrap and return result, or log and exit if Err.
macro_rules! unwrap_or_exit {
    ($f:expr, $origin:tt) => {
        match $f {
            Ok(a) => a,
            Err(e) => {
                error!("{}: {}", $origin, e);
                process::exit(1);
            }
        }
    };
}

pub fn main() {
    let cli_input = init_cli();
    init_logger(cli_input.verbose);

    // Lexer
    let lexer = unwrap_or_exit!(Lexer::from_file(&cli_input.input_path), "IO");
    let tokens = lexer
        .map(|t| unwrap_or_exit!(t, "Lexing"))
        .collect::<Vec<_>>();

    if cli_input.print_tokens {
        println!("***TOKENS***");
        tokens.iter().for_each(|t| println!("{:?}", t));
    }

    // Parser
    let mut parser = Parser::new(tokens.into_iter().peekable());
    let program = unwrap_or_exit!(parser.parse_program(), "Parsing");
    if cli_input.print_ast {
        println!("***AST***\n{:#?}", program);
    }

    // Generator
    let generator = unsafe { Generator::new(program, &cli_input.input_name) };
    unsafe {
        unwrap_or_exit!(generator.generate(), "Code Generation");
        unwrap_or_exit!(generator.verify(), "LLVM");
    }

    match cli_input.output_format {
        OutputFormat::LLVM => unsafe {
            unwrap_or_exit!(generator.generate_ir(&cli_input.output_path), "LLVM");
        },
        OutputFormat::ObjectFile => unsafe {
            unwrap_or_exit!(
                generator.generate_object_file(cli_input.optimization, &cli_input.output_path),
                "LLVM"
            );
        },
        OutputFormat::Executable => unsafe {
            let object_file = format!("{}.o", cli_input.input_name);
            unwrap_or_exit!(
                generator.generate_object_file(cli_input.optimization, &object_file),
                "LLVM"
            );
            unwrap_or_exit!(
                generator.generate_executable(&object_file, &cli_input.output_path),
                "Linker"
            );
            fs::remove_file(object_file).unwrap_or_else(|e| {
                warn!("Unable to delete object file:\n{}", e);
            });
        },
    }
}
