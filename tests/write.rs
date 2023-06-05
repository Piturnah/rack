use clap::Parser;
use std::path::Path;
use test_utils::{run_test, Mode};

#[derive(Parser)]
struct Config {
    /// Write the output of a specific test case or all test cases.
    #[clap(short, long)]
    write: Option<Option<String>>,
}

fn main() {
    if let Some(to_write) = Config::parse().write {
        match to_write {
            Some(file) => {
                let file = format!("tests/src/{file}.rk");
                let file = Path::new(&file);
                let file_path = file.to_str().unwrap();
                let file_stem = file.file_stem().unwrap().to_str().unwrap();
                run_test(&file_path, Mode::Write);
                println!("Updated expected output for test case `{file_stem}`")
            }
            None => {
                std::fs::read_dir("tests/src")
                    .unwrap()
                    .map(|file| {
                        std::thread::spawn(|| {
                            let file = file.unwrap().path();
                            let file_stem = file.file_stem().unwrap().to_str().unwrap();
                            let file_path = file.to_str().unwrap();
                            run_test(file_path, Mode::Write);
                            println!("Updated expected output for test case `{file_stem}`")
                        })
                    })
                    .for_each(|t| t.join().unwrap());
            }
        }
    }
}
