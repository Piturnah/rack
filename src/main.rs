use clap::Parser;
use compiler::Program;
use std::{
    error::Error,
    fmt, fs,
    process::{self, Stdio},
    str::FromStr,
};

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

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::X86_64_Linux => write!(f, "x86_64-linux"),
            Self::X86_64_FASM => write!(f, "x86_64-FASM"),
            Self::Mos6502_Nesulator => write!(f, "mos_6502-nesulator"),
        }
    }
}

impl FromStr for Target {
    type Err = TargetNotFoundError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "x86_64-linux" => Ok(Self::X86_64_Linux),
            "X86_64-fasm" => Ok(Self::X86_64_FASM),
            "mos_6502-nesulator" => Ok(Self::Mos6502_Nesulator),
            _ => Err(TargetNotFoundError),
        }
    }
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
}

fn main() {
    let config = Config::parse();

    let source_f = &config.file;
    let source = match fs::read_to_string(source_f) {
        Ok(source) => source,
        Err(_) => {
            eprintln!("Couldn't read file `{}`", source_f);
            process::exit(1);
        }
    };

    let program = Program::parse(&source, &config.file);

    match config.target {
        Target::X86_64_Linux => {
            println!("[INFO] Generating `./out.asm`");

            let outbuf = program.generate_fasm_x86_64_linux();
            fs::write("./out.asm", &outbuf).expect("Unable to write to out.asm");

            run_command("fasm out.asm");

            run_command("chmod +x out");

            if config.run {
                run_command("./out");
            }
        }
        Target::X86_64_FASM => {
            println!("[INFO] Genereting `./out.asm`");
            let outbuf = program.generate_fasm_x86_64_linux();
            fs::write("./out.asm", &outbuf).expect("failed to write to out.asm");
        }
        Target::Mos6502_Nesulator => {
            println!("[INFO] Generating `./out`");

            let outbuf = program.generate_code_mos_6502_nesulator();
            fs::write("./out", &outbuf).expect("Unable to write to ./out");

            println!("[INFO] Wrote {} bytes", outbuf.len());
        }
    }
}

fn run_command(cmd: &str) {
    println!("[INFO] Running `{}`", cmd);
    let mut cmd = cmd.split_whitespace();

    match process::Command::new(cmd.next().expect("No command provided"))
        .args(cmd.collect::<Vec<&str>>())
        .stdout(Stdio::inherit())
        .output()
    {
        Ok(_) => {}
        Err(e) => {
            eprintln!("[ERROR] {}", e);
            process::exit(1);
        }
    };
}
