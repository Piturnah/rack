use std::process;

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

#[derive(Debug)]
pub enum Op {
    PushInt(u64),
    Plus,
    Minus,
    Dup,
    Drop,
    Equals,
    GreaterThan,
    LessThan,
    Or,
    And,
    If(Option<usize>),
    While(Option<Box<Op>>),
    End(Box<Op>),
    Print,
}

pub fn parse_program(source: &str) -> Vec<Op> {
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
                    "drop" => program.push(Op::Drop),
                    "=" => program.push(Op::Equals),
                    ">" => program.push(Op::GreaterThan),
                    "<" => program.push(Op::LessThan),
                    "or" => program.push(Op::Or),
                    "and" => program.push(Op::And),
                    "if" => {
                        ret_stack.push(program.len());
                        program.push(Op::If(None));
                    }
                    "while" => {
                        ret_stack.push(program.len());
                        program.push(Op::While(None));
                    }
                    "do" => {
                        let index = *ret_stack
                            .get(ret_stack.len() - 1)
                            .expect("Empty return stack");
                        match program[index] {
                            Op::While(None) => {
                                program[index] = Op::While(Some(Box::new(Op::If(Some(jmp_count)))));
                                program.push(Op::If(Some(jmp_count)));
                                jmp_count += 2;
                            }
                            _ => unreachable!(),
                        }
                    }
                    "end" => {
                        let index = ret_stack.pop().expect("Empty return stack");
                        match &program[index] {
                            Op::If(None) => {
                                program[index] = Op::If(Some(jmp_count));
                                program.push(Op::End(Box::new(Op::If(Some(jmp_count)))));
                                jmp_count += 1;
                            }
                            Op::While(Some(op)) => match **op {
                                Op::If(Some(id)) => {
                                    program.push(Op::End(Box::new(Op::While(Some(Box::new(
                                        Op::If(Some(id)),
                                    ))))));
                                }
                                _ => unreachable!(),
                            },
                            _ => unreachable!(),
                        };
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

pub fn generate_fasm_x86_64(program: Vec<Op>) -> String {
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
            Op::Drop => {
                outbuf = outbuf
                    + "  ;; Op::Drop
  pop rax
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
                        "  ;; Op::LessThan
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
                eprintln!("[ERROR] No closing `end` for `if` keyword");
                process::exit(1);
            }
            Op::While(Some(op)) => match *op {
                Op::If(Some(id)) => {
                    outbuf = outbuf
                        + &format!(
                            "  ;; Op::While
F{}:
",
                            id + 1
                        )
                }
                _ => unreachable!(),
            },
            Op::While(None) => {
                eprintln!("[ERROR] No closing `do` for `while` keyword");
                process::exit(1);
            }
            Op::End(op) => match *op {
                Op::If(Some(id)) => {
                    outbuf = outbuf
                        + &format!(
                            "  ;; Op::End(If)
F{}:
",
                            id
                        );
                }
                Op::If(None) => unreachable!(),
                Op::While(Some(op)) => match *op {
                    Op::If(Some(id)) => {
                        outbuf = outbuf
                            + &format!(
                                " ;; Op::End(While)
  jmp F{}
F{}:
",
                                id + 1,
                                id
                            )
                    }
                    _ => unreachable!(),
                },
                Op::While(None) => unreachable!(),
                Op::Dup
                | Op::Drop
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

    outbuf
        + "  mov rax, 60
  mov rdi, 0
  syscall"
}
