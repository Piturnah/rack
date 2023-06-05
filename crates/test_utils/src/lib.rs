use std::{
    fs::{self, DirBuilder},
    path::Path,
    process::Command,
};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Mode {
    Write,
    Compare,
}

pub fn run_test(file: &str, mode: Mode) {
    let path = Path::new(file);
    // Panic if the file doesn't exist.
    fs::metadata(path).unwrap();
    let file_stem = path.file_stem().unwrap().to_str().unwrap();
    let build_path = format!("tests/build/{}", file_stem,);
    DirBuilder::new()
        .recursive(true)
        .create("tests/build")
        .unwrap();
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
                println!("==============EXPECTED==============");
                println!(
                    "{expected}",
                    expected = std::str::from_utf8(&expected).unwrap()
                );
                println!("===============ACTUAL===============");
                println!("{output}", output = std::str::from_utf8(&output).unwrap());
                panic!("program `{file}` did not match its expected output!");
            }
        }
    }
}
