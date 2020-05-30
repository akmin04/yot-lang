use log::{error, warn};
use std::{fs, process};
use yotc::generator::Generator;
use yotc::lexer::Lexer;
use yotc::parser::Parser;
use yotc::{init_cli, init_logger, OutputFormat};

pub fn main() {
    let cli_input = init_cli();
    init_logger(cli_input.verbose);

    // Lexer
    let lexer = Lexer::from_file(&cli_input.input_path).unwrap_or_else(|e| {
        error!("IO Error: {}", e);
        process::exit(1);
    });

    let tokens = lexer
        .map(|t| {
            t.unwrap_or_else(|e| {
                error!("Lexing Error: {}", e);
                process::exit(1);
            })
        })
        .collect::<Vec<_>>();

    if cli_input.print_tokens {
        println!("***TOKENS***");
        tokens.iter().for_each(|t| println!("{:?}", t));
    }

    // Parser
    let mut parser = Parser::new(tokens.into_iter().peekable());
    let program = match parser.parse_program() {
        Ok(e) => e,
        Err(e) => {
            error!("Parsing Error: {}", e);
            process::exit(1);
        }
    };
    if cli_input.print_ast {
        println!("***AST***\n{:#?}", program);
    }

    // Generator
    let generator = Generator::new(program, &cli_input.input_name);
    if let Err(e) = generator.generate() {
        error!("Code Generation Error: {}", e);
        process::exit(1)
    };
    match cli_input.output_format {
        OutputFormat::LLVM => generator.generate_ir(&cli_input.output_path),
        OutputFormat::ObjectFile => {
            generator.generate_object_file(cli_input.optimization, &cli_input.output_path)
        }
        OutputFormat::Executable => {
            let object_file = format!("{}.o", cli_input.input_name);
            generator.generate_object_file(cli_input.optimization, &object_file);
            if let Err(e) = generator.generate_executable(&object_file, &cli_input.output_path) {
                error!("Linking Error: {}", e);
                process::exit(1);
            };
            fs::remove_file(object_file).unwrap_or_else(|e| {
                warn!("Unable to delete object file:\n{}", e);
            });
        }
    }
}
