#![warn(clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::option_if_let_else,
    clippy::missing_const_for_fn,
    clippy::must_use_candidate,
    clippy::needless_pass_by_value,
    clippy::explicit_iter_loop
)]

use clap::Parser;
use std::{
    error::Error,
    fmt, fs,
    path::Path,
    process::{self, Stdio},
    str::FromStr,
};

pub use crate::{
    lex::Lexer,
    parse::{parse_tokens, Context, Func, Op, Program},
};

mod codegen;
mod lex;
mod parse;

#[allow(non_camel_case_types)]
enum Target {
    X86_64_Linux,
    X86_64_FASM,
    Mos6502_Nesulator,
}

impl Default for Target {
    fn default() -> Self {
        Self::X86_64_Linux
    }
}

macro_rules! target_as_str {
    {$($var:tt => $string:literal),*,} => {
        impl fmt::Display for Target {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    $(Self::$var => write!(f, $string)),*,
                }
            }
        }

        impl FromStr for Target {
            type Err = TargetNotFoundError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_lowercase().as_str() {
                    $($string => Ok(Self::$var)),*,
                    _ => Err(TargetNotFoundError),
                }
            }
        }
    }
}

target_as_str! {
    X86_64_Linux => "x86_64-linux",
    X86_64_FASM => "x86_64-fasm",
    Mos6502_Nesulator => "mos_6502-nesulator",
}

#[derive(Debug)]
struct TargetNotFoundError;

impl Error for TargetNotFoundError {}

impl fmt::Display for TargetNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "target not supported")
    }
}

#[derive(Parser)]
struct Config {
    /// Run the program after successful compilation
    #[clap(short, long)]
    run: bool,
    /// Target architecture
    #[clap(short, long, default_value_t)]
    target: Target,
    /// Input file
    #[clap()]
    file: String,
    /// Output file
    #[clap(short, long, value_name = "FILE")]
    out: Option<String>,
    /// Don't print log information
    #[clap(short, long)]
    quiet: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::parse();

    let source_f = &config.file;
    let Ok(source) = fs::read_to_string(source_f) else {
        eprintln!("Couldn't read file `{source_f}`");
        process::exit(1);
    };

    let mut lexer = Lexer::new(&source, Some(source_f));
    let program = parse::parse_tokens(&mut lexer).unwrap_or_else(|e| {
        eprintln!("{e}");
        process::exit(1);
    });
    // TODO: do this properly.
    if !program.funcs.iter().any(|f| f.ident == "main") {
        eprintln!("[ERROR] No entry point `main` found.");
        process::exit(1);
    }

    // Determine output path of compiled program
    let default_path = &config.out.clone().unwrap_or_else(|| {
        match config.target {
            Target::X86_64_Linux | Target::Mos6502_Nesulator => "./out",
            Target::X86_64_FASM => "./out.asm",
        }
        .to_string()
    });
    let output_path = Path::new(&default_path);

    match config.target {
        Target::X86_64_Linux => {
            let asm_path = output_path
                .with_extension("asm")
                .to_str()
                .expect("path name is invalid unicode")
                .to_owned();

            if !config.quiet {
                println!("[INFO] Generating `{asm_path}`");
            }

            let outbuf = codegen::fasm_x86_64_linux::generate(program)?;
            fs::write(&asm_path, outbuf)
                .unwrap_or_else(|_| panic!("failed to write to {asm_path}"));

            run_command(&format!("fasm {asm_path}"), &config, !config.quiet);

            let output_path = output_path
                .to_str()
                .expect("path name is invalid unicode")
                .to_owned();
            run_command(&format!("chmod +x {output_path}"), &config, !config.quiet);

            if config.run {
                run_command(&output_path, &config, true);
            }

            Ok(())
        }
        Target::X86_64_FASM => {
            let output_path = output_path
                .to_str()
                .expect("path name is invalid unicode")
                .to_owned();
            if !config.quiet {
                println!("[INFO] Generating `{output_path}`");
            }
            let outbuf = codegen::fasm_x86_64_linux::generate(program)?;
            fs::write(&output_path, outbuf)
                .unwrap_or_else(|_| panic!("failed to write to {output_path}"));
            Ok(())
        }
        Target::Mos6502_Nesulator => {
            let output_path = output_path
                .to_str()
                .expect("path name is invalid unicode")
                .to_owned();
            if !config.quiet {
                println!("[INFO] Generating `{output_path}`");
            }

            let outbuf = codegen::mos_6502_nesulator::generate(program);
            fs::write(&output_path, outbuf)
                .unwrap_or_else(|_| panic!("Unable to write to {output_path}"));

            if !config.quiet {
                println!("[INFO] Wrote {} bytes", outbuf.len());
            }

            Ok(())
        }
    }
}

fn run_command(cmd: &str, config: &Config, echo: bool) {
    if !config.quiet {
        println!("[INFO] Running `{cmd}`");
    }
    let mut cmd = cmd.split_whitespace();

    let out_pipe = if echo { Stdio::inherit } else { Stdio::null };
    match process::Command::new(cmd.next().expect("No command provided"))
        .args(cmd)
        .stdout(out_pipe())
        .stderr(out_pipe())
        .output()
    {
        Ok(output) => {
            if let Some(code) = output.status.code() {
                if code != 0 {
                    process::exit(code)
                }
            }
        }
        Err(e) => {
            eprintln!("[ERROR] {e}");
            process::exit(1);
        }
    }
}
