use clap::Parser;
use rackc::Program;
use std::{
    error::Error,
    fmt, fs,
    path::Path,
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
        match s.to_lowercase().as_str() {
            "x86_64-linux" => Ok(Self::X86_64_Linux),
            "x86_64-fasm" => Ok(Self::X86_64_FASM),
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

    /// Output file
    #[clap(short, long, value_name = "FILE")]
    out: Option<String>,
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

    // Determine output path of compiled program
    let default_path = &config.out.unwrap_or_else(|| {
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

            println!("[INFO] Generating `{}`", asm_path);

            let outbuf = program.generate_fasm_x86_64_linux();
            fs::write(&asm_path, &outbuf).expect("Unable to write to out.asm");

            run_command(&format!("fasm {}", asm_path));

            let output_path = output_path
                .to_str()
                .expect("path name is invalid unicode")
                .to_owned();
            run_command(&format!("chmod +x {}", output_path));

            if config.run {
                run_command(&output_path);
            }
        }
        Target::X86_64_FASM => {
            let output_path = output_path
                .to_str()
                .expect("path name is invalid unicode")
                .to_owned();
            println!("[INFO] Generating `{}`", &output_path);
            let outbuf = program.generate_fasm_x86_64_linux();
            fs::write(&output_path, &outbuf)
                .unwrap_or_else(|_| panic!("failed to write to {}", output_path));
        }
        Target::Mos6502_Nesulator => {
            let output_path = output_path
                .to_str()
                .expect("path name is invalid unicode")
                .to_owned();
            println!("[INFO] Generating `{}`", &output_path);

            let outbuf = program.generate_code_mos_6502_nesulator();
            fs::write(&output_path, &outbuf)
                .unwrap_or_else(|_| panic!("Unable to write to {}", output_path));

            println!("[INFO] Wrote {} bytes", outbuf.len());
        }
    }
}

fn run_command(cmd: &str) {
    println!("[INFO] Running `{}`", cmd);
    let mut cmd = cmd.split_whitespace();

    if let Err(e) = process::Command::new(cmd.next().expect("No command provided"))
        .args(cmd.collect::<Vec<&str>>())
        .stdout(Stdio::inherit())
        .output()
    {
        eprintln!("[ERROR] {}", e);
        process::exit(1);
    };
}
