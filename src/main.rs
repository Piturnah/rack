use clap::Parser;
use compiler::Program;
use std::{
    fs,
    process::{self, Stdio},
};

#[derive(Parser)]
struct Config {
    /// Run the program after successful compilation
    #[clap(short, long)]
    run: bool,

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

    println!("[INFO] Generating `out.asm`");

    let outbuf = program.generate_fasm_x86_64();
    fs::write("./out.asm", &outbuf).expect("Unable to write to out.asm");

    run_command("fasm out.asm");

    run_command("chmod +x out");

    if config.run {
        run_command("./out");
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
