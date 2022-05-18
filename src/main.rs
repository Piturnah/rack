use std::env;
use std::fs;
use std::process;

#[derive(Debug)]
enum Op {
    PushInt(u64),
    Plus,
    Minus,
    Print,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        error("No target file provided.");
    }

    let source_f = &args[1];
    let source = match fs::read_to_string(source_f) {
        Ok(source) => source,
        Err(_) => {
            error(&format!("Couldn't read file {}", source_f));
            String::from("")
        }
    };

    // Parse source file into vector of ops
    let mut stack: Vec<Op> = Vec::new();
    for line in source.split("\n") {
        for word in line.split(" ") {
            if let Ok(val) = word.parse::<u64>() {
                stack.push(Op::PushInt(val));
            } else if word == "+" {
                stack.push(Op::Plus);
            } else if word == "-" {
                stack.push(Op::Minus);
            } else if word == "print" {
                stack.push(Op::Print);
            }
        }
    }

    // Compile into fasm_f86-64
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

    for op in stack {
        outbuf = match op {
            Op::PushInt(val) => outbuf + &format!("mov rax, {}\npush rax\n", val),
            Op::Plus => outbuf + &format!("pop rax\npop rbx\nadd rax, rbx\npush rax\n"),
            Op::Minus => outbuf + &format!("pop rbx\npop rax\nsub rax, rbx\npush rax\n"),
            Op::Print => outbuf + &format!("pop rdi\ncall print\n"),
        };
    }

    outbuf = outbuf
        + "mov rax, 60
mov rdi, 0
syscall";
    fs::write("./out.asm", &outbuf).expect("Unable to write to out.asm");
}

fn usage() {
    let args: Vec<String> = env::args().collect();
    println!("Usage: {} <filename>", args[0]);
}

fn error(msg: &str) {
    eprintln!("Error: {}", msg);
    usage();
    process::exit(1);
}
