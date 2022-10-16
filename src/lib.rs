use std::{
    collections::HashMap,
    fmt::{self, Write},
    process,
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

pub struct Token<'l> {
    op: Op,
    loc: Loc<'l>,
}

impl<'l> Token<'l> {
    fn new(op: Op, loc: Loc<'l>) -> Self {
        Token { op, loc }
    }
}

#[derive(Clone, Copy)]
struct Loc<'a> {
    file: &'a str,
    row: usize,
    col: usize,
}

impl<'a> Loc<'a> {
    fn new(file: &'a str, row: usize, col: usize) -> Self {
        Loc { file, row, col }
    }
}

impl fmt::Display for Loc<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}:{}:{}", self.file, self.row, self.col)
    }
}

#[derive(Debug, PartialEq)]
pub enum Op {
    PushInt(u64),
    PushStrPtr(usize),
    PushBinding(usize),
    Let,
    In,
    Plus,
    Minus,
    Dup,
    Drop,
    Swap,
    Equals,
    Neq,
    Not,
    GreaterThan,
    LessThan,
    Or,
    And,
    If(Option<usize>),
    While(Option<Box<Op>>),
    End(Box<Op>),
    Print,
    Puts, // Later move to stdlib?
}

pub struct Program<'a> {
    program: Vec<Token<'a>>,
    string_literals: Vec<String>,
}

impl<'a> Program<'a> {
    pub fn parse(source: &str, path: &'a str) -> Self {
        let mut program: Vec<Token> = Vec::new();
        let mut ret_stack: Vec<usize> = Vec::new();
        let mut bindings: HashMap<String, usize> = HashMap::new();
        let mut jmp_count = 0;

        let mut string_literals: Vec<String> = Vec::new();

        'lines: for (row, line) in source.split('\n').enumerate() {
            // I think this implementation of getting the col of each word is kind of ugly
            // Because it involves a lot of allocations which *should* be unecessary.
            // TODO: Better implementation without allocations
            let mut cs = line.char_indices().peekable();
            let mut ws: Vec<(usize, String)> = Vec::new();

            while cs.peek().is_some() {
                if cs.peek().unwrap().1 != ' ' {
                    let c = cs.next().unwrap();
                    match c.1 {
                        '"' => {
                            let mut parsing_string = String::new();
                            loop {
                                match cs.next() {
                                    Some((_, char)) => {
                                        if char == '"' {
                                            ws.push((
                                                c.0,
                                                format!("IR_LIT_STR_{}", string_literals.len()),
                                            ));
                                            string_literals.push(parsing_string);
                                            break;
                                        } else if char == '\\' {
                                            match cs.next().unwrap() {
                                                (_, 'n') => parsing_string += "\n",
                                                (_, '"') => parsing_string += "\"",
                                                (_, c) => parsing_string += &c.to_string(),
                                            }
                                        } else {
                                            parsing_string += &char.to_string();
                                        }
                                    }
                                    None => {
                                        eprintln!(
                                            "{path}:{}:{}: No closing `\"` for string literal",
                                            row + 1,
                                            c.0 + 1
                                        );
                                        process::exit(1);
                                    }
                                }
                            }
                        }
                        _ => {
                            ws.push((c.0, c.1.to_string()));

                            while cs.peek().is_some() && cs.peek().unwrap().1 != ' ' {
                                let last_ws_i = ws.len() - 1;
                                ws[last_ws_i] = (
                                    ws[last_ws_i].0,
                                    ws.get(last_ws_i).unwrap().1.clone()
                                        + &cs.next().unwrap().1.to_string(),
                                );
                            }
                        }
                    }
                }

                cs.next();
            }

            for (col, word) in ws {
                let loc = Loc::new(path, row + 1, col + 1);
                macro_rules! push {
                    ($op:expr) => {
                        program.push(Token::new($op, loc))
                    };
                }

                if let Ok(val) = word.parse::<u64>() {
                    push!(Op::PushInt(val));
                } else if let Some(word) = word.strip_prefix("0x") {
                    push!(Op::PushInt(match u64::from_str_radix(word, 16) {
                        Ok(val) => val,
                        Err(e) => {
                            eprintln!("{loc}: Could not parse hex literal: {e}.");
                            process::exit(1);
                        }
                    }));
                } else if let Some(str_index) = word.strip_prefix("IR_LIT_STR_") {
                    let str_index = str_index.parse::<usize>().unwrap();
                    push!(Op::PushInt(
                        string_literals.get(str_index).unwrap().len() as u64
                    ));
                    push!(Op::PushStrPtr(str_index));
                } else {
                    match &word[..] {
                        "+" => push!(Op::Plus),
                        "-" => push!(Op::Minus),
                        "dup" => push!(Op::Dup),
                        "drop" => push!(Op::Drop),
                        "swap" => push!(Op::Swap),
                        "=" => push!(Op::Equals),
                        "!=" => push!(Op::Neq),
                        "not" => push!(Op::Not),
                        ">" => push!(Op::GreaterThan),
                        "<" => push!(Op::LessThan),
                        "or" => push!(Op::Or),
                        "and" => push!(Op::And),
                        "if" => {
                            ret_stack.push(program.len());
                            push!(Op::If(None));
                        }
                        "while" => {
                            ret_stack.push(program.len());
                            push!(Op::While(None));
                        }
                        "do" => {
                            let index = match ret_stack.last() {
                                Some(index) => *index,
                                None => {
                                    eprintln!("{}: `do` must close `while` condn", loc);
                                    process::exit(1);
                                }
                            };
                            match program[index].op {
                                Op::While(None) => {
                                    program[index].op =
                                        Op::While(Some(Box::new(Op::If(Some(jmp_count)))));
                                    push!(Op::If(Some(jmp_count)));
                                    jmp_count += 2;
                                }
                                _ => unreachable!(),
                            }
                        }
                        "let" => {
                            ret_stack.push(program.len());
                            bindings = HashMap::new();
                            push!(Op::Let);
                        }
                        "in" => {
                            if let Some(index) = ret_stack.pop() {
                                if program[index].op == Op::Let {
                                    ret_stack.push(program.len());
                                    push!(Op::In);
                                    continue;
                                }
                            }
                            eprintln!("{loc}: `in` must close `let` binding");
                            process::exit(1);
                        }
                        "end" => {
                            let index = match ret_stack.pop() {
                                Some(index) => index,
                                None => {
                                    eprintln!(
                                        "{}: `end` must close either `do` or `if` block",
                                        loc
                                    );
                                    process::exit(1);
                                }
                            };
                            match &program[index].op {
                                Op::If(None) => {
                                    program[index].op = Op::If(Some(jmp_count));
                                    push!(Op::End(Box::new(Op::If(Some(jmp_count)))));
                                    jmp_count += 1;
                                }
                                Op::While(Some(op)) => match **op {
                                    Op::If(Some(id)) => {
                                        push!(Op::End(Box::new(Op::While(Some(Box::new(
                                            Op::If(Some(id)),
                                        ))))));
                                    }
                                    _ => unreachable!(),
                                },
                                Op::In => {}
                                op => {
                                    eprintln!("{loc}: `end` keyword doesn't support closing `{op:?}` blocks");
                                    process::exit(1);
                                }
                            };
                        }
                        "true" => push!(Op::PushInt(1)),
                        "false" => push!(Op::PushInt(0)),
                        "print" => push!(Op::Print),
                        "puts" => push!(Op::Puts),
                        "//" => continue 'lines,
                        "" => {}
                        _ => {
                            if let Some(index) = ret_stack.last() {
                                if program[*index].op == Op::Let {
                                    bindings.insert(word, bindings.len());
                                    continue;
                                }
                                if let Some(index) = bindings.get(&word) {
                                    push!(Op::PushBinding(bindings.len() - *index - 1));
                                    continue;
                                }
                            }
                            eprintln!("{}: Unknown word `{}` in program source", loc, word);
                            process::exit(1);
                        }
                    }
                }
            }
        }
        Self {
            program,
            string_literals,
        }
    }

    pub fn generate_fasm_x86_64(self) -> String {
        let program = self.program;

        let mut outbuf = String::from(ASM_HEADER);

        let mut jump_target_count = 0;

        for (loc, op) in program.into_iter().map(|tok| (tok.loc, tok.op)) {
            match op {
                Op::PushInt(val) => {
                    outbuf = outbuf
                        + &format!(
                            "  ;; Op::PushInt({0}) - {1}
  mov rax, {0}
  push rax
",
                            val, loc
                        );
                }
                Op::PushStrPtr(index) => {
                    outbuf = outbuf
                        + &format!(
                            "  ;; Op::PushStrPtr({0}) - {1}
  push str_{0}
",
                            index, loc
                        );
                }
                Op::PushBinding(index) => {
                    outbuf = outbuf
                        + &format!(
                            "  ;; Op::PushBinding({0}) - {1}
  mov rbx, rsp
  mov rsp, r9
  add rsp, {2}
  pop rax
  mov rsp, rbx
  push rax
",
                            index,
                            loc,
                            index * 8,
                        );
                }
                Op::Plus => {
                    outbuf = outbuf
                        + &format!(
                            "  ;; Op::Plus - {}
  pop rax
  pop rbx
  add rax, rbx
  push rax
",
                            loc
                        );
                }
                Op::Minus => {
                    outbuf = outbuf
                        + &format!(
                            "  ;; Op::Minus - {}
  pop rbx
  pop rax
  sub rax, rbx
  push rax
",
                            loc
                        );
                }
                Op::Dup => {
                    outbuf = outbuf
                        + &format!(
                            "  ;; Op::Dup - {}
  pop rax
  push rax
  push rax
",
                            loc
                        );
                }
                Op::Drop => {
                    outbuf = outbuf
                        + &format!(
                            "  ;; Op::Drop - {}
  add rsp, 8
",
                            loc
                        );
                }
                Op::Swap => {
                    outbuf = outbuf
                        + &format!(
                            "  ;; Op::Swap - {}
  pop rax
  pop rbx
  push rax
  push rbx
",
                            loc
                        );
                }
                Op::Equals => {
                    outbuf = outbuf
                        + &format!(
                            "  ;; Op::Equals - {loc}
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
                            jump_target_count + 1,
                        );
                    jump_target_count += 2;
                }
                Op::Neq => {
                    outbuf = outbuf
                        + &format!(
                            "  ;; Op::Neq - {loc}
  pop rax
  pop rbx
  cmp rax, rbx
  jne J{0}
  push 0
  jmp J{1}
J{0}:
  push 1
J{1}:
",
                            jump_target_count,
                            jump_target_count + 1,
                        );
                    jump_target_count += 2;
                }
                Op::Not => {
                    outbuf = outbuf
                        + &format!(
                            "  ;; Op::Not - {loc}
  pop rax
  mov rbx, 1
  sub rbx, rax
  push rbx
"
                        )
                }
                Op::GreaterThan => {
                    outbuf = outbuf
                        + &format!(
                            "  ;; Op::GreaterThan - {loc}
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
                            jump_target_count + 1,
                        );
                    jump_target_count += 2;
                }
                Op::LessThan => {
                    outbuf = outbuf
                        + &format!(
                            "  ;; Op::LessThan - {loc}
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
                            jump_target_count + 1,
                        );
                    jump_target_count += 2;
                }
                Op::Or => {
                    outbuf = outbuf
                        + &format!(
                            "  ;; Op::Or - {loc}
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
                            jump_target_count + 1,
                        );
                    jump_target_count += 2;
                }
                Op::And => {
                    outbuf = outbuf
                        + &format!(
                            "  ;; Op::And - {2}
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
                            jump_target_count + 1,
                            loc
                        );
                    jump_target_count += 2;
                }
                Op::If(Some(jump_to)) => {
                    outbuf = outbuf
                        + &format!(
                            "  ;; Op::If - {loc}
  pop rax
  cmp rax, 1
  jne F{}
",
                            jump_to
                        )
                }
                Op::If(None) => {
                    eprintln!("{}: No closing `end` for `if` keyword", loc);
                    process::exit(1);
                }
                Op::While(Some(op)) => match *op {
                    Op::If(Some(id)) => {
                        outbuf = outbuf
                            + &format!(
                                "  ;; Op::While - {loc}
F{}:
",
                                id + 1
                            )
                    }
                    _ => unreachable!(),
                },
                Op::While(None) => {
                    eprintln!("{}: No closing `do` for `while` keyword", loc);
                    process::exit(1);
                }
                Op::End(op) => match *op {
                    Op::If(Some(id)) => {
                        outbuf = outbuf
                            + &format!(
                                "  ;; Op::End(If) - {loc}
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
                                    " ;; Op::End(While) - {loc}
  jmp F{}
F{}:
",
                                    id + 1,
                                    id
                                )
                        }
                        _ => unreachable!(),
                    },
                    Op::In => todo!("codegen for Op::In closing End"),
                    Op::While(None) => unreachable!(),
                    Op::Dup
                    | Op::Drop
                    | Op::Equals
                    | Op::Swap
                    | Op::Neq
                    | Op::Not
                    | Op::GreaterThan
                    | Op::LessThan
                    | Op::Minus
                    | Op::Plus
                    | Op::Print
                    | Op::Puts
                    | Op::PushInt(_)
                    | Op::PushStrPtr(_)
                    | Op::PushBinding(_)
                    | Op::Let
                    | Op::Or
                    | Op::And
                    | Op::End(_) => unreachable!("End block cannot close `{:?}`", op),
                },
                Op::Print => {
                    outbuf = outbuf + &format!("  ;; Op::Print - {loc}\n  pop rdi\n  call print\n")
                }
                Op::Puts => {
                    outbuf = outbuf
                        + &format!(
                            "  ;; Op::Puts - {loc} 
  mov rdi, 1
  pop rsi
  pop rdx
  mov rax, 1
  syscall
"
                        )
                }
                Op::Let => {
                    outbuf = outbuf
                        + &format!(
                            "  ;; Op::Let - {loc}
  mov r9, rsp
"
                        )
                }
                Op::In => outbuf = outbuf + &format!("  ;; Op::In\n"),
            };
        }
        outbuf += "  mov rax, 60
  mov rdi, 0
  syscall
segment readable
";
        for (i, s) in self.string_literals.iter().enumerate() {
            let mut s_bytes = String::new();
            for b in s.as_bytes() {
                write!(&mut s_bytes, "{b},").unwrap();
            }
            outbuf += &format!("str_{i}: db {}\n", s_bytes.trim_end_matches(","))
        }

        outbuf
    }
}
