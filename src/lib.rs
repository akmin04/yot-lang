pub mod generator;
pub mod lexer;
pub mod parser;

use clap::{App, Arg};
use log::LevelFilter;
use std::path;

pub type Result<T> = std::result::Result<T, String>;

/// Output file format.
pub enum OutputFormat {
    /// LLVM Intermediate Representation.
    LLVM,
    /// Unlinked object file.
    ObjectFile,
    /// Object file linked with `gcc`.
    Executable,
}

/// CLI input configuration and parameters.
pub struct CLIInput {
    /// Path to `.yot` input file.
    pub input_path: String,
    /// `input_path` file name without file extension.
    pub input_name: String,
    /// Path to output file.
    pub output_path: String,
    /// Format of output file.
    pub output_format: OutputFormat,
    /// Optimization level (0-3)
    pub optimization: u32,
    /// Whether or not raw tokens should be printed.
    pub print_tokens: bool,
    /// Whether or not raw AST should be printed.
    pub print_ast: bool,
    /// Whether to filter logs or not.
    pub verbose: u32,
}

/// Initialize command line application to parse arguments.
pub fn init_cli() -> CLIInput {
    let matches = App::new("yotc")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Compiler for yot lang - a toy language")
        .arg(
            Arg::with_name("input")
                .help("Path to the yot file")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("output")
                .help("Path to generated output")
                .takes_value(true)
                .short("o")
                .long("output"),
        )
        .arg(
            Arg::with_name("output format")
                .help("The type of file to output")
                .takes_value(true)
                .possible_values(&["llvm", "executable", "object-file"])
                .default_value("executable")
                .short("f")
                .long("output-format"),
        )
        .arg(
            Arg::with_name("optimization")
                .help("Level of optimization")
                .takes_value(true)
                .use_delimiter(false)
                .possible_values(&["0", "1", "2", "3"])
                .default_value("2")
                .short("O")
                .long("optimization"),
        )
        .arg(
            Arg::with_name("print tokens")
                .help("Print raw tokens from the lexer")
                .long("print-tokens"),
        )
        .arg(
            Arg::with_name("print AST")
                .help("Print the raw abstract syntax tree")
                .long("print-ast"),
        )
        .arg(
            Arg::with_name("verbose")
                .help("Level of logging (0-2)")
                .short("v")
                .multiple(true),
        )
        .get_matches();

    let input_path = matches.value_of("input").unwrap();
    let input_name = path::Path::new(input_path)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    let output_format = match matches.value_of("output format").unwrap_or("executable") {
        "llvm" => OutputFormat::LLVM,
        "object-file" => OutputFormat::ObjectFile,
        "executable" => OutputFormat::Executable,
        _ => panic!("Unhandled output format"),
    };
    let default_output_path = format!(
        "{}.{}",
        input_name,
        match output_format {
            OutputFormat::LLVM => "ll",
            OutputFormat::ObjectFile => "o",
            OutputFormat::Executable => "out",
        }
    );

    CLIInput {
        input_path: String::from(input_path),
        input_name: String::from(input_name),
        output_path: String::from(matches.value_of("output").unwrap_or(&default_output_path)),
        output_format,
        optimization: matches.value_of("optimization").unwrap().parse().unwrap(),
        print_tokens: matches.is_present("print tokens"),
        print_ast: matches.is_present("print AST"),
        verbose: matches.occurrences_of("verbose") as u32,
    }
}

/// Initialize logger with verbosity filter.
pub fn init_logger(verbose: u32) {
    env_logger::builder()
        .format_timestamp(None)
        .format_module_path(false)
        .filter_level(match verbose {
            0 => LevelFilter::Warn,
            1 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        })
        .init()
}
