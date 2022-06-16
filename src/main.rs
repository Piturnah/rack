use clap::Parser;
use compiler::*;
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

    let program = parse_program(&source);

    println!("[INFO] Generating `out.asm`");

    let outbuf = generate_fasm_x86_64(program);
    fs::write("./out.asm", &outbuf).expect("Unable to write to out.asm");

    println!("[INFO] Running `fasm out.asm`");
    match process::Command::new("fasm")
        .args(["out.asm"])
        .stdout(Stdio::inherit())
        .output()
    {
        Ok(_) => {}
        Err(e) => {
            eprintln!("[ERROR] {}", e);
            process::exit(1);
        }
    };

    if config.run {
        println!("[INFO] Running `./out`");
        match process::Command::new("./out")
            .stdout(Stdio::inherit())
            .output()
        {
            Ok(_) => {}
            Err(e) => {
                eprintln!("[ERROR] {}", e);
                process::exit(1);
            }
        }
    }
}
