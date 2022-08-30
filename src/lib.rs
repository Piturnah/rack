use std::{
    collections::HashMap,
    fmt::{self, Write},
    process,
};

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

#[derive(Debug)]
pub enum Op {
    PushInt(u64),
    PushStrPtr(usize),
    Plus,
    Minus,
    Dup,
    Drop,
    Swap,
    Over,
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
    Puts,          // Later move to stdlib?
    Bind(u64),     // number of variables to bind
    PushBind(u64), // index of binding to push
}

pub struct Program<'a> {
    program: Vec<Token<'a>>,
    string_literals: Vec<String>,
}

impl<'a> Program<'a> {
    pub fn parse(source: &str, path: &'a str) -> Self {
        let mut program: Vec<Token> = Vec::new();
        let mut ret_stack: Vec<usize> = Vec::new();
        let mut jmp_count = 0;

        let mut let_stack: Vec<Op> = Vec::new();
        let mut bindings: Vec<HashMap<String, u64>> = Vec::new();

        let mut string_literals: Vec<String> = Vec::new();

        'lines: for (row, line) in source.lines().enumerate() {
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
                } else if let Some(word) = word.strip_prefix("0o") {
                    push!(Op::PushInt(match u64::from_str_radix(word, 8) {
                        Ok(val) => val,
                        Err(e) => {
                            eprintln!("{loc}: Could not parse octal literal: {e}.");
                            process::exit(1);
                        }
                    }));
                } else if let Some(word) = word.strip_prefix("0b") {
                    push!(Op::PushInt(match u64::from_str_radix(word, 2) {
                        Ok(val) => val,
                        Err(e) => {
                            eprintln!("{loc}: Could not parse binary literal: {e}.");
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
                        "over" => push!(Op::Over),
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
                                Op::Bind(_) => {
                                    bindings.pop();
                                }
                                op => {
                                    eprintln!("{loc}: `end` tried to close `{op:?}`.");
                                    eprintln!(
                                        "{}: [NOTE] `{op:?}` found here.",
                                        program[index].loc
                                    );
                                    process::exit(1);
                                }
                            };
                        }
                        "true" => push!(Op::PushInt(1)),
                        "false" => push!(Op::PushInt(0)),
                        "print" => push!(Op::Print),
                        "puts" => push!(Op::Puts),
                        "let" => {
                            let_stack.push(Op::Bind(0));
                            bindings.push(HashMap::new());
                        }
                        "in" => {
                            match let_stack.pop() {
                                Some(Op::Bind(count)) => {
                                    ret_stack.push(program.len());
                                    push!(Op::Bind(count));
                                }
                                _ => {
                                    eprintln!("{loc}: `in` must close `let` bindings");
                                    process::exit(1);
                                }
                            };
                        }
                        "//" => continue 'lines,
                        "" => {}
                        w => {
                            if let Some(Op::Bind(count)) = let_stack.pop() {
                                let_stack.push(Op::Bind(count + 1));
                                let count = bindings.iter().map(|map| map.len() as u64).sum();
                                bindings.last_mut().unwrap().insert(w.to_string(), count);
                            } else if let Some(index) =
                                bindings.last().unwrap_or(&HashMap::new()).get(w)
                            {
                                push!(Op::PushBind(*index));
                            } else if w.starts_with('\'') && w.ends_with('\'') {
                                match w.len() {
                                    3 => {
                                        push!(Op::PushInt(w.chars().nth(1).unwrap() as u64));
                                    }
                                    _ => {
                                        eprintln!("{loc}: Character literals must be only one char")
                                    }
                                }
                            } else {
                                eprintln!("{}: Unknown word `{}` in program source", loc, word);
                                process::exit(1);
                            }
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

    pub fn generate_fasm_x86_64_linux(self) -> String {
        let program = self.program;

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

        let mut jump_target_count = 0;

        for (loc, op) in program.into_iter().map(|tok| (tok.loc, tok.op)) {
            match op {
                Op::Bind(count) => {
                    outbuf = outbuf
                        + &format!(
                            "  ;; Op::Bind({}) - {loc}
  mov rax, ret_stack_rsp
  sub rax, {}
  mov [ret_stack_rsp], rax
",
                            count,
                            count * 8
                        );
                    for i in 0..count {
                        outbuf = outbuf
                            + &format!(
                                "  pop rbx
  mov [rax+{}], rbx
",
                                (count - 1 - i) * 8
                            );
                    }
                }
                Op::PushBind(index) => {
                    outbuf = outbuf
                        + &format!(
                            "  ;; Op::PushBind({index}) - {loc}
  mov rax, [ret_stack_rsp]
  add rax, {}
  push qword [rax]
",
                            index * 8
                        )
                }
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
                Op::Over => {
                    outbuf = outbuf
                        + &format!(
                            "  ;; Op::Over - {loc}
  pop rax
  pop rbx
  pop rcx
  push rbx
  push rax
  push rcx
"
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
                    Op::While(None) => unreachable!(),
                    Op::Dup
                    | Op::Bind(_)
                    | Op::PushBind(_)
                    | Op::Drop
                    | Op::Equals
                    | Op::Swap
                    | Op::Over
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
            outbuf += &format!("str_{i}: db {}\n", s_bytes.trim_end_matches(','))
        }

        outbuf
            + "segment readable writable
ret_stack_rsp: rq 1
ret_stack: rb 65536
ret_stack_end:
"
    }

    pub fn generate_code_mos_6502_nesulator(self) -> [u8; 65536 - 0x4020] {
        const NOP: u8 = 0xea;
        const PHA: u8 = 0x48;
        const PLA: u8 = 0x68;
        const CLC: u8 = 0x18;
        const SEC: u8 = 0x38;
        const ADC_ZPG: u8 = 0x65;
        const SBC_ZPG: u8 = 0xe5;
        const LDA_IMM: u8 = 0xa9;
        const LDA_ZPG: u8 = 0xa5;
        const STA_ZPG: u8 = 0x85;
        const BNE: u8 = 0xd0;
        const CMP_IMM: u8 = 0xc9;
        const CMP_ZPG: u8 = 0xc5;

        let mut outbuf = vec![NOP; 65536 - 0x4020];
        outbuf[0xfffc - 0x4020] = 0x20;
        outbuf[0xfffd - 0x4020] = 0x40;

        let mut pc: usize = 0x00;

        let mut unclosed_ifs = Vec::new();
        let mut unclosed_whiles = Vec::new();

        for (loc, op) in self.program.into_iter().map(|tok| (tok.loc, tok.op)) {
            let opcodes = match op {
                Op::PushInt(val) => vec![LDA_IMM, val as u8, PHA],
                Op::Plus => vec![PLA, STA_ZPG, 0x00, PLA, CLC, ADC_ZPG, 0x00, PHA],
                Op::Minus => vec![PLA, STA_ZPG, 0x00, PLA, SEC, SBC_ZPG, 0x00, PHA],
                Op::Drop => vec![PLA],
                Op::Over => vec![
                    PLA, STA_ZPG, 0x00, PLA, STA_ZPG, 0x01, PLA, STA_ZPG, 0x02, LDA_ZPG, 0x01, PHA,
                    LDA_ZPG, 0x02, PHA, LDA_ZPG, 0x00, PHA,
                ],
                Op::Dup => vec![PLA, PHA, PHA],
                Op::Neq => vec![
                    PLA, STA_ZPG, 0x00, PLA, CMP_ZPG, 0x00, BNE, 0x09, LDA_IMM, 0x00, PHA, LDA_IMM,
                    0x01, BNE, 0x05, LDA_IMM, 0x01, PHA,
                ],
                Op::Swap => vec![
                    PLA, STA_ZPG, 0x00, PLA, STA_ZPG, 0x01, LDA_ZPG, 0x00, PHA, LDA_ZPG, 0x01, PHA,
                ],
                Op::If(_) => {
                    unclosed_ifs.push(pc + 3);
                    vec![PLA, CMP_IMM, 0x01, BNE, 0x00]
                }
                Op::While(_) => {
                    unclosed_whiles.push(pc);
                    vec![]
                }
                Op::End(op) => match *op {
                    Op::If(_) => {
                        let branch_index = unclosed_ifs
                            .pop()
                            .expect("`end` has no opening keyword in codegen!");
                        outbuf[branch_index as usize + 1] = (pc - branch_index) as i8 as u8;
                        vec![]
                    }
                    Op::While(_) => {
                        let branch_index = unclosed_ifs
                            .pop()
                            .expect("`end` has no opening keyword in codegen!");
                        outbuf[branch_index as usize + 1] = (pc + 4 - branch_index) as i8 as u8;

                        let while_index = unclosed_whiles
                            .pop()
                            .expect("`endwhile` has no opening `while` in codegen!");

                        vec![
                            LDA_IMM,
                            0x01,
                            BNE,
                            (while_index.wrapping_sub(pc)) as i8 as u8,
                        ]
                    }
                    _ => todo!(),
                },
                op => {
                    eprintln!("{loc}: `{op:?}` not implemented in codegen!");
                    process::exit(1);
                }
            };
            outbuf.splice(pc..pc + opcodes.len(), opcodes.iter().cloned());
            pc += opcodes.len();
        }

        outbuf.try_into().unwrap_or_else(|v: Vec<u8>| {
            panic!(
                "Expected Vec of length {} but it was {}",
                65536 - 0x4020,
                v.len()
            )
        })
    }
}
