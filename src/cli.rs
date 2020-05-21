use clap::{App, Arg};
use log::LevelFilter;
use std::path;

pub enum OutputFormat {
    LLVM,
    ObjectFile,
    Executable,
}

pub struct CLIInput {
    pub input_path: String,
    pub input_name: String,
    pub output_path: String,
    pub output_format: OutputFormat,
    pub optimization: u32,
    pub print_tokens: bool,
    pub print_ast: bool,
    pub verbose: bool,
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
                .help("Whether or not verbose logging should be displayed")
                .short("v")
                .long("verbose"),
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
        verbose: matches.is_present("verbose"),
    }
}

/// Initialize logger with verbosity filter.
pub fn init_logger(verbose: bool) {
    env_logger::builder()
        .format_timestamp(None)
        .format_module_path(false)
        .filter_level(if verbose {
            LevelFilter::Trace
        } else {
            LevelFilter::Warn
        })
        .init()
}
