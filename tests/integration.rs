use std::{
    fs::{self, read_dir, DirBuilder, DirEntry},
    process::Command,
    thread,
};

#[derive(Copy, Clone, PartialEq, Eq)]
enum Mode {
    Write,
    Compare,
}

fn run_test(file: DirEntry, mode: Mode) {
    let path = file.path();
    let file_stem = path.file_stem().unwrap().to_str().unwrap();
    let build_path = format!("tests/build/{}", file_stem,);
    let child = Command::new("cargo")
        .args([
            "r",
            "-q",
            "--",
            "-qr",
            path.as_os_str().to_str().unwrap(),
            "-o",
            &build_path,
        ])
        .output()
        .unwrap();

    DirBuilder::new()
        .recursive(true)
        .create("tests/expected")
        .unwrap();
    let output_path = &format!("tests/expected/{}.out", file_stem);
    let output: Vec<_> = b"----STDOUT----\n"
        .iter()
        .chain(child.stdout.iter())
        .chain(b"\n----STDERR----\n".iter())
        .chain(child.stderr.iter())
        .map(|b| *b)
        .collect();

    match mode {
        Mode::Write => fs::write(output_path, output).unwrap(),
        Mode::Compare => {
            let expected = fs::read(output_path).unwrap();
            if output != expected {
                println!("--------------EXPECTED--------------");
                println!(
                    "{expected}",
                    expected = std::str::from_utf8(&expected).unwrap()
                );
                println!("---------------ACTUAL---------------");
                println!("{output}", output = std::str::from_utf8(&output).unwrap());
                panic!(
                    "program `{}` did not match its expected output!",
                    file.file_name().to_string_lossy()
                );
            }
        }
    }
}

fn main() {
    DirBuilder::new()
        .recursive(true)
        .create("tests/build")
        .unwrap();
    let mode = if std::env::args().any(|arg| arg == "write") {
        Mode::Write
    } else {
        Mode::Compare
    };

    read_dir("tests/src")
        .unwrap()
        .map(|file| thread::spawn(move || run_test(file.unwrap(), mode)))
        .for_each(|t| t.join().unwrap());
}
