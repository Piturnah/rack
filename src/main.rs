use clap::Parser;
use std::{
    fs,
    process::{self, Stdio},
};

const ASM_HEADER: &str = "format ELF64 executable 3
entry main
segment readable executable
print:
  mov     r9, -3689348814741910323
  sub     rsp, 40
  mov     BYTE [rsp+31], 10
  lea     rcx, [rsp+30]
.L2:
  mov     rax, rdi
  lea     r8, [rsp+32]
  mul     r9
  mov     rax, rdi
  sub     r8, rcx
  shr     rdx, 3
  lea     rsi, [rdx+rdx*4]
  add     rsi, rsi
  sub     rax, rsi
  add     eax, 48
  mov     BYTE [rcx], al
  mov     rax, rdi
  mov     rdi, rdx
  mov     rdx, rcx
  sub     rcx, 1
  cmp     rax, 9
  ja      .L2
  lea     rax, [rsp+32]
  mov     edi, 1
  sub     rdx, rax
  xor     eax, eax
  lea     rsi, [rsp+32+rdx]
  mov     rdx, r8
  mov     rax, 1
  syscall
  add     rsp, 40
  ret
main:\n";

#[derive(Parser)]
struct Config {
    /// Run the program after successful compilation
    #[clap(short, long)]
    run: bool,

    /// Input file
    #[clap()]
    file: String,
}

#[derive(Debug)]
enum Op {
    PushInt(u64),
    Plus,
    Minus,
    Dup,
    Equals,
    GreaterThan,
    LessThan,
    Or,
    And,
    If(Option<usize>),
    End(Box<Op>),
    Print,
}

fn parse_program(source: &str) -> Vec<Op> {
    let mut program: Vec<Op> = Vec::new();
    let mut ret_stack: Vec<usize> = Vec::new();
    let mut jmp_count = 0;

    'lines: for line in source.split("\n") {
        for word in line.split(" ") {
            if let Ok(val) = word.parse::<u64>() {
                program.push(Op::PushInt(val));
            } else {
                match word {
                    "+" => program.push(Op::Plus),
                    "-" => program.push(Op::Minus),
                    "dup" => program.push(Op::Dup),
                    "=" => program.push(Op::Equals),
                    ">" => program.push(Op::GreaterThan),
                    "<" => program.push(Op::LessThan),
                    "or" => program.push(Op::Or),
                    "and" => program.push(Op::And),
                    "if" => {
                        ret_stack.push(program.len());
                        program.push(Op::If(None));
                    }
                    "end" => {
                        let index = ret_stack.pop().expect("Empty return stack");
                        match program[index] {
                            Op::If(None) => {
                                program[index] = Op::If(Some(jmp_count));
                                program.push(Op::End(Box::new(Op::If(Some(jmp_count)))));
                            }
                            _ => unreachable!(),
                        };
                        jmp_count += 1;
                    }
                    "true" => program.push(Op::PushInt(1)),
                    "false" => program.push(Op::PushInt(0)),
                    "print" => program.push(Op::Print),
                    "//" => continue 'lines,
                    "" => {}
                    _ => {
                        eprintln!("[ERROR] Unknown word `{}` in program source", word);
                        process::exit(1);
                    }
                }
            }
        }
    }
    program
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

    // Compile into fasm_x86-64
    let mut outbuf = String::from(ASM_HEADER);

    let mut jump_target_count = 0;

    for op in program {
        match op {
            Op::PushInt(val) => {
                outbuf = outbuf + &format!("  ;; Op::PushInt({})\n  push {}\n", val, val);
            }
            Op::Plus => {
                outbuf =
                    outbuf + "  ;; Op::Plus\n  pop rax\n  pop rbx\n  add rax, rbx\n  push rax\n";
            }
            Op::Minus => {
                outbuf =
                    outbuf + "  ;; Op::Minus\n  pop rbx\n  pop rax\n  sub rax, rbx\n  push rax\n";
            }
            Op::Dup => {
                outbuf = outbuf
                    + "  ;; Op::Dup
  pop rax
  push rax
  push rax
"
            }
            Op::Equals => {
                outbuf = outbuf
                    + &format!(
                        "  ;; Op::Equals
  pop rax
  pop rbx
  cmp rax, rbx
  je J{0}
  push 0
  jmp J{1}
J{0}:
  push 1
J{1}:
",
                        jump_target_count,
                        jump_target_count + 1
                    );
                jump_target_count += 2;
            }
            Op::GreaterThan => {
                outbuf = outbuf
                    + &format!(
                        "  ;; Op::GreaterThan
  pop rax
  pop rbx
  cmp rax, rbx
  jb J{0}
  push 0
  jmp J{1}
J{0}:
  push 1
J{1}:
",
                        jump_target_count,
                        jump_target_count + 1
                    );
                jump_target_count += 2;
            }
            Op::LessThan => {
                outbuf = outbuf
                    + &format!(
                        "  ;; Op::GreaterThan
  pop rax
  pop rbx
  cmp rbx, rax
  jb J{0}
  push 0
  jmp J{1}
J{0}:
  push 1
J{1}:
",
                        jump_target_count,
                        jump_target_count + 1
                    );
                jump_target_count += 2;
            }
            Op::Or => {
                outbuf = outbuf
                    + &format!(
                        "  ;; Op::Or
  pop rax
  pop rbx
  cmp rax, 1
  je J{0}
  cmp rbx, 1
  je J{0}
  push 0
  jmp J{1}
J{0}:
  push 1
J{1}:
",
                        jump_target_count,
                        jump_target_count + 1
                    );
                jump_target_count += 2;
            }
            Op::And => {
                outbuf = outbuf
                    + &format!(
                        "  ;; Op::And
  pop rax
  pop rbx
  cmp rax, rbx
  jne J{0}
  cmp rax, 1
  jne J{0}
  push 1
  jmp J{1}
J{0}:
  push 0
J{1}:
",
                        jump_target_count,
                        jump_target_count + 1
                    );
                jump_target_count += 2;
            }
            Op::If(Some(jump_to)) => {
                outbuf = outbuf
                    + &format!(
                        "  ;; Op::If
  pop rax
  cmp rax, 1
  jne F{}
",
                        jump_to
                    )
            }
            Op::If(None) => {
                eprintln!("No closing `end` for `if` keyword");
                process::exit(1);
            }
            Op::End(op) => match *op {
                Op::If(Some(id)) => {
                    outbuf = outbuf
                        + &format!(
                            "  ;; Op::End
F{}:
",
                            id
                        );
                }
                Op::If(None) => unreachable!(),
                Op::Dup
                | Op::Equals
                | Op::GreaterThan
                | Op::LessThan
                | Op::Minus
                | Op::Plus
                | Op::Print
                | Op::PushInt(_)
                | Op::Or
                | Op::And
                | Op::End(_) => {
                    eprintln!("[ERROR] End block cannot close `{:?}`", op);
                    process::exit(1);
                }
            },
            Op::Print => outbuf = outbuf + "  ;; Op::Print\n  pop rdi\n  call print\n",
        };
    }

    outbuf = outbuf
        + "  mov rax, 60
  mov rdi, 0
  syscall";

    println!("[INFO] Generating `out.asm`");
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
