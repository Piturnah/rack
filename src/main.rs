use std::{env, fs, process};

#[derive(Debug)]
enum Op {
    PushInt(u64),
    Plus,
    Minus,
    Print,
}

fn parse_program(source: &str) -> Vec<Op> {
    let mut program: Vec<Op> = Vec::new();
    'lines: for line in source.split("\n") {
       for word in line.split(" ") {
           if let Ok(val) = word.parse::<u64>() {
               program.push(Op::PushInt(val));
           } else {
               match word {
                   "+" => program.push(Op::Plus),
                   "-" => program.push(Op::Minus),
                   "print" => program.push(Op::Print),
                   "//" => continue 'lines,
                   _ => {
                       eprintln!("Unknown word `{}` in program source", word);
                       process::exit(1);
                   }
               }
           }
       }
    }
    program
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("No target file provided.");
        usage();
        process::exit(1);
    }

    let source_f = &args[1];
    let source = match fs::read_to_string(source_f) {
        Ok(source) => source,
        Err(_) => {
            eprintln!("Couldn't read file `{}`", source_f);
            process::exit(1);
        }
    };

    let program = parse_program(&source);

    // Compile into fasm_x86-64
    let mut outbuf = String::from(
        "format ELF64 executable 3
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
main:\n",
    );

    for op in program {
        outbuf = match op {
            Op::PushInt(val) => {
                outbuf
                    + &format!(
                        "  ;; Op::PushInt({})\n  mov rax, {}\n  push rax\n",
                        val, val
                    )
            }
            Op::Plus => {
                outbuf
                    + &format!("  ;; Op::Plus\n  pop rax\n  pop rbx\n  add rax, rbx\n  push rax\n")
            }
            Op::Minus => {
                outbuf
                    + &format!("  ;; Op::Minus\n  pop rbx\n  pop rax\n  sub rax, rbx\n  push rax\n")
            }
            Op::Print => outbuf + &format!("  ;; Op::Print\n  pop rdi\n  call print\n"),
        };
    }

    outbuf = outbuf
        + "  mov rax, 60
  mov rdi, 0
  syscall";
    fs::write("./out.asm", &outbuf).expect("Unable to write to out.asm");
}

fn usage() {
    let args: Vec<String> = env::args().collect();
    println!("Usage: {} <filename>", args[0]);
}
