#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::too_many_lines,
    clippy::similar_names,
    clippy::missing_panics_doc
)]

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

#[derive(Debug, PartialEq)]
pub enum Op {
    PushInt(u64),
    PushStrPtr(usize),
    Plus,
    Minus,
    DivMod,
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
    ReadByte,
    If(Option<usize>),
    While(Option<Box<Op>>),
    End(Box<Op>),
    Print,
    Func(usize), // usize -- jump index
    CallFn(usize),
    Ret(usize),        // the number of stack frames to drop
    Puts,              // Later move to stdlib?
    Bind(usize, bool), // (number of variables to bind, are we peeking)
    PushBind(usize),   // index of binding to push
    Unbind(usize),
}

pub struct Program<'a> {
    entry: usize, // jmp index of the FN to jump to at entry
    program: Vec<Token<'a>>,
    string_literals: Vec<String>,
}

impl<'a> Program<'a> {
    #[must_use]
    pub fn parse(source: &str, path: &'a str) -> Self {
        let mut program: Vec<Token> = Vec::new();

        // lets, whiles, ifs
        // contains indexes into `program` that we need to jump back into
        let mut ret_stack: Vec<usize> = Vec::new();

        // increments with each thing we may need to jump to in codegen
        let mut jmp_count = 0;

        let mut let_stack: Vec<Op> = Vec::new();
        let mut bindings: Vec<Vec<String>> = Vec::new();

        let mut string_literals: Vec<String> = Vec::new();

        //                          name    jmp
        let mut functions: HashMap<String, usize> = HashMap::new();
        let mut parsing_function_def: bool = false;

        'lines: for (row, line) in source.lines().enumerate() {
            // I think this implementation of getting the col of each word is kind of ugly
            // Because it involves a lot of allocations which *should* be unecessary.
            // TODO: Better implementation without allocations
            let mut cs = line.char_indices().peekable();
            let mut ws: Vec<(usize, String)> = Vec::new();

            while cs.peek().is_some() {
                if cs.peek().unwrap().1 != ' ' {
                    let c = cs.next().unwrap();
                    if c.1 == '"' {
                        let mut parsing_string = String::new();
                        loop {
                            let Some((_, char)) = cs.next() else {
                                eprintln!(
                                    "{path}:{}:{}: No closing `\"` for string literal",
                                    row + 1,
                                    c.0 + 1
                                );
                                process::exit(1);
                            };
                            if char == '"' {
                                ws.push((c.0, format!("IR_LIT_STR_{}", string_literals.len())));
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
                    } else {
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

                cs.next();
            }

            for (col, word) in ws {
                let loc = Loc::new(path, row + 1, col + 1);
                macro_rules! push {
                    ($($op:expr),+) => {{
                        $(program.push(Token::new($op, loc)));+
                    }};
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
                        "divmod" => push!(Op::DivMod),
                        "/" => push!(Op::DivMod, Op::Drop),
                        "%" => push!(Op::DivMod, Op::Swap, Op::Drop),
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
                        "@" => push!(Op::ReadByte),
                        "if" => {
                            ret_stack.push(program.len());
                            push!(Op::If(None));
                        }
                        "fn" => {
                            parsing_function_def = true;
                        }
                        "ret" => push!(Op::Ret(bindings.iter().flatten().count())),
                        "while" => {
                            ret_stack.push(program.len());
                            push!(Op::While(None));
                        }
                        "do" => {
                            let Some(&index) = ret_stack.last() else {
                                eprintln!("{loc}: `do` must close `while` condn");
                                process::exit(1);
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
                            let Some(index) = ret_stack.pop() else {
                                eprintln!("{loc}: `end` must close either `do` or `if` block");
                                process::exit(1);
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
                                Op::Bind(count, _) => {
                                    bindings.pop();
                                    push!(Op::Unbind(*count));
                                }
                                Op::Func(_) => {
                                    push!(Op::Ret(0));
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
                            let_stack.push(Op::Bind(0, false));
                            bindings.push(Vec::new());
                        }
                        "peek" => {
                            let_stack.push(Op::Bind(0, true));
                            bindings.push(Vec::new());
                        }
                        "in" => {
                            match let_stack.pop() {
                                Some(Op::Bind(count, is_peek)) => {
                                    ret_stack.push(program.len());
                                    push!(Op::Bind(count, is_peek));
                                }
                                _ => {
                                    if parsing_function_def {
                                        ret_stack.push(program.len());
                                        push!(Op::Func(functions.len() - 1));
                                        parsing_function_def = false;
                                    } else {
                                        eprintln!("{loc}: `in` must close `let`/`peek` bindings or function definitions.");
                                        process::exit(1);
                                    }
                                }
                            };
                        }
                        "//" => continue 'lines,
                        "" => {}
                        w => {
                            if let Some(Op::Bind(count, is_peek)) = let_stack.pop() {
                                let_stack.push(Op::Bind(count + 1, is_peek));

                                bindings.last_mut().unwrap().push(w.to_string());
                            } else if let Some(index) =
                                bindings.iter().flatten().rev().position(|x| x == w)
                            {
                                push!(Op::PushBind(index));
                            } else if w.starts_with('\'') && w.ends_with('\'') {
                                match w.len() {
                                    3 => {
                                        push!(Op::PushInt(w.chars().nth(1).unwrap() as u64));
                                    }
                                    _ => {
                                        eprintln!(
                                            "{loc}: Character literals must be only one char"
                                        );
                                    }
                                }
                            } else if parsing_function_def {
                                functions.insert(w.to_string(), functions.len());
                            } else if let Some(jmp_index) = functions.get(w) {
                                push!(Op::CallFn(*jmp_index));
                            } else {
                                eprintln!("{loc}: Unknown word `{word}` in program source");
                                process::exit(1);
                            }
                        }
                    }
                }
            }
        }
        Self {
            entry: *functions
                .iter()
                .find(|(name, _)| *name == "main")
                .map_or_else(
                    || {
                        eprintln!("[ERROR] No entry point `main` found.");
                        process::exit(1);
                    },
                    |(_, index)| index,
                ),
            program,
            string_literals,
        }
    }

    #[must_use]
    pub fn generate_fasm_x86_64_linux(self) -> String {
        let program = self.program;

        let mut outbuf = String::from(
            "format ELF64 executable 3
entry main
segment readable executable
",
        );

        if program.iter().any(|t| t.op == Op::Print) {
            outbuf += "print:
\tmov\tr9, -3689348814741910323
\tsub\trsp, 40
\tmov\tBYTE [rsp+31], 10
\tlea\trcx, [rsp+30]
.L2:
\tmov\trax, rdi
\tlea\tr8, [rsp+32]
\tmul\tr9
\tmov\trax, rdi
\tsub\tr8, rcx
\tshr\trdx, 3
\tlea\trsi, [rdx+rdx*4]
\tadd\trsi, rsi
\tsub\trax, rsi
\tadd\teax, 48
\tmov\tBYTE [rcx], al
\tmov\trax, rdi
\tmov\trdi, rdx
\tmov\trdx, rcx
\tsub\trcx, 1
\tcmp\trax, 9
\tja\t.L2
\tlea\trax, [rsp+32]
\tmov\tedi, 1
\tsub\trdx, rax
\txor\teax, eax
\tlea\trsi, [rsp+32+rdx]
\tmov\trdx, r8
\tmov\trax, 1
\tsyscall
\tadd\trsp, 40
\tret
";
        }

        let mut jump_target_count = 0;

        for (i, (loc, op)) in program.into_iter().map(|tok| (tok.loc, tok.op)).enumerate() {
            #[allow(clippy::match_same_arms)]
            match op {
                Op::Func(index) => outbuf.push_str(&format!(
                    "FN{index}:\t\t\t\t\t; Op::Func({index})\t\t{loc}\n"
                )),
                Op::CallFn(index) => outbuf.push_str(&format!(
                    "\tmov\trax, [ret_stack_rsp]\t; Op::CallFn({index})\t\t{loc}
\tsub\trax, 8
\tmov\t[ret_stack_rsp], rax
\tmov\tqword [rax], RET{i}
\tjmp\tFN{index}
RET{i}:
\tmov\trax, [ret_stack_rsp]
\tadd\trax, 8
\tmov\t[ret_stack_rsp], rax
"
                )),
                Op::Ret(0) => outbuf.push_str(&format!(
                    "\tmov\trax, qword [ret_stack_rsp]\t; Op::Ret(0)\t\t{loc}
\tjmp qword [rax]
"
                )),
                Op::Ret(count) => outbuf.push_str(&format!(
                    "\tmov\trax, [ret_stack_rsp]\t; Op::Ret({count})\t\t{loc}
\tadd\trax, {}
\tmov\tqword [ret_stack_rsp], rax
\tjmp\tqword [rax]
",
                    count * 8
                )),
                Op::Bind(count, is_peek) => {
                    outbuf.push_str(&format!(
                        "\tmov\trax, [ret_stack_rsp]\t; Op::Bind({})\t\t{loc}
\tsub\trax, {}
\tmov\t[ret_stack_rsp], rax
",
                        count,
                        count * 8
                    ));
                    for i in 0..count {
                        outbuf.push_str(&format!(
                            "\tmov\trbx, [rsp + {0}]
\tmov\t[rax+{0}], rbx
",
                            i * 8
                        ));
                    }
                    if !is_peek {
                        outbuf.push_str(&format!("\tadd\trsp, {}\n", count * 8));
                    }
                }
                Op::Unbind(count) => outbuf.push_str(&format!(
                    "\tmov\trax, [ret_stack_rsp]\t; Op::Unbind({count})\t\t{loc}
\tadd\trax, {}
\tmov\tqword [ret_stack_rsp], rax
",
                    count * 8
                )),
                Op::PushBind(index) => outbuf.push_str(&format!(
                    "\tmov\trax, [ret_stack_rsp]\t; Op::PushBind({index})\t{loc}
\tadd\trax, {}
\tpush\tqword [rax]
",
                    index * 8
                )),
                Op::PushInt(val) => outbuf.push_str(&format!(
                    "\tpush\t{0}\t\t\t; Op::PushInt({0})\t{1}\n",
                    val, loc
                )),
                Op::PushStrPtr(index) => outbuf.push_str(&format!(
                    "\tpush\tstr_{0}\t\t\t; Op::PushStrPtr({0})\t{1}\n",
                    index, loc
                )),
                Op::Plus => outbuf.push_str(&format!(
                    "\tpop\trax\t\t\t; Op::Plus\t\t{loc}
\tpop\trbx
\tadd\trax, rbx
\tpush\trax
",
                )),
                Op::Minus => outbuf.push_str(&format!(
                    "\tpop\trbx\t\t\t; Op::Minus\t\t{loc}
\tpop\trax
\tsub\trax, rbx
\tpush\trax
",
                )),
                Op::DivMod => outbuf.push_str(&format!(
                    "\tpop\trbx\t\t\t; Op::DivMod\t\t{loc}
\tpop\trax
\tmov\trdx, 0
\tdiv\trbx
\tpush\trax
\tpush\trdx
",
                )),
                Op::Dup => {
                    outbuf.push_str(&format!("\tpush\tqword [rsp]\t\t; Op::Dup\t\t{loc}\n",));
                }
                Op::Drop => outbuf.push_str(&format!("\tadd\trsp, 8\t\t\t; Op::Drop\t\t{loc}\n",)),
                Op::Swap => outbuf.push_str(&format!(
                    "\tpop\trax\t\t\t; Op::Swap\t\t{loc}
\tpop\trbx
\tpush\trax
\tpush\trbx
",
                )),
                Op::Over => outbuf.push_str(&format!(
                    "\tpop\trax\t\t\t; Op::Over\t\t{loc}
\tpop\trbx
\tpop\trcx
\tpush\trbx
\tpush\trax
\tpush\trcx
"
                )),
                Op::Equals => {
                    outbuf.push_str(&format!(
                        "\tpop\trax\t\t\t; Op::Equals\t\t{loc}
\tpop\trbx
\tcmp\trax, rbx
\tje\tJ{0}
\tpush\t0
\tjmp\tJ{1}
J{0}:
\tpush\t1
J{1}:
",
                        jump_target_count,
                        jump_target_count + 1,
                    ));
                    jump_target_count += 2;
                }
                Op::Neq => {
                    outbuf.push_str(&format!(
                        "\tpop\trax\t\t\t; Op::Neq\t\t{loc}
\tpop\trbx
\tcmp\trax, rbx
\tjne\tJ{0}
\tpush\t0
\tjmp\tJ{1}
J{0}:
\tpush\t1
J{1}:
",
                        jump_target_count,
                        jump_target_count + 1,
                    ));
                    jump_target_count += 2;
                }
                Op::Not => outbuf.push_str(&format!(
                    "\tpop\trax\t\t\t; Op::Not\t\t{loc}
\tmov\trbx, 1
\tsub\trbx, rax
\tpush\trbx
"
                )),
                Op::GreaterThan => {
                    outbuf.push_str(&format!(
                        "\tpop\trax\t\t\t; Op::GreaterThan\t{loc}
\tpop\trbx
\tcmp\trax, rbx
\tjb\tJ{0}
\tpush\t0
\tjmp\tJ{1}
J{0}:
\tpush\t1
J{1}:
",
                        jump_target_count,
                        jump_target_count + 1,
                    ));
                    jump_target_count += 2;
                }
                Op::LessThan => {
                    outbuf.push_str(&format!(
                        "\tpop\trax\t\t\t; Op::LessThan\t\t{loc}
\tpop\trbx
\tcmp\trbx, rax
\tjb\tJ{0}
\tpush\t0
\tjmp\tJ{1}
J{0}:
\tpush\t1
J{1}:
",
                        jump_target_count,
                        jump_target_count + 1,
                    ));
                    jump_target_count += 2;
                }
                Op::Or => {
                    outbuf.push_str(&format!(
                        "\tpop\trax\t\t\t; Op::Or\t\t{loc}
\tpop\trbx
\tcmp\trax, 1
\tje\tJ{0}
\tcmp\trbx, 1
\tje\tJ{0}
\tpush\t0
\tjmp\tJ{1}
J{0}:
\tpush\t1
J{1}:
",
                        jump_target_count,
                        jump_target_count + 1,
                    ));
                    jump_target_count += 2;
                }
                Op::And => {
                    outbuf.push_str(&format!(
                        "\tpop\trax\t\t\t; Op::And\t\t{2}
\tpop\trbx
\tcmp\trax, rbx
\tjne\tJ{0}
\tcmp\trax, 1
\tjne\tJ{0}
\tpush\t1
\tjmp\tJ{1}
J{0}:
\tpush\t0
J{1}:
",
                        jump_target_count,
                        jump_target_count + 1,
                        loc
                    ));
                    jump_target_count += 2;
                }
                Op::ReadByte => outbuf.push_str(&format!(
                    "\tpop\trbx\t\t\t; Op::ReadByte\t\t{loc}
\tmov\trax, 0
\tmov\tal, byte [rbx]
\tpush\trax
"
                )),

                Op::If(Some(jump_to)) => outbuf.push_str(&format!(
                    "\tpop\trax\t\t\t; Op::If\t\t{loc}
\tcmp\trax, 1
\tjne\tF{jump_to}
"
                )),
                Op::If(None) => {
                    eprintln!("{loc}: No closing `end` for `if` keyword");
                    process::exit(1);
                }
                Op::While(Some(op)) => match *op {
                    Op::If(Some(id)) => outbuf.push_str(&format!(
                        "F{}:\t\t\t\t\t; Op::While\t\t{loc}
",
                        id + 1
                    )),
                    _ => unreachable!(),
                },
                Op::While(None) => {
                    eprintln!("{loc}: No closing `do` for `while` keyword");
                    process::exit(1);
                }
                Op::End(op) => match *op {
                    Op::If(Some(id)) => {
                        outbuf.push_str(&format!("F{id}:\t\t\t\t\t; Op::End(If)\t\t{loc}\n",));
                    }
                    Op::If(None) => unreachable!(),
                    Op::While(Some(op)) => match *op {
                        Op::If(Some(id)) => outbuf.push_str(&format!(
                            "\tjmp\tF{}\t\t\t; Op::End(While)\t{loc}
F{}:
",
                            id + 1,
                            id
                        )),
                        _ => unreachable!(),
                    },
                    Op::While(None) => unreachable!(),
                    Op::Dup
                    | Op::Bind(_, _)
                    | Op::PushBind(_)
                    | Op::Unbind(_)
                    | Op::Drop
                    | Op::Equals
                    | Op::Func(_)
                    | Op::CallFn(_)
                    | Op::ReadByte
                    | Op::Ret(_)
                    | Op::Swap
                    | Op::Over
                    | Op::Neq
                    | Op::Not
                    | Op::GreaterThan
                    | Op::LessThan
                    | Op::Minus
                    | Op::Plus
                    | Op::DivMod
                    | Op::Print
                    | Op::Puts
                    | Op::PushInt(_)
                    | Op::PushStrPtr(_)
                    | Op::Or
                    | Op::And
                    | Op::End(_) => unreachable!("End block cannot close `{:?}`", op),
                },
                Op::Print => outbuf.push_str(&format!(
                    "\tpop\trdi\t\t\t; Op::Print\t\t{loc}\n\tcall\tprint\n"
                )),
                Op::Puts => outbuf.push_str(&format!(
                    "\tmov\trdi, 1\t\t\t; Op::Puts\t\t{loc}
\tpop\trsi
\tpop\trdx
\tmov\trax, 1
\tsyscall
"
                )),
            };
        }
        outbuf += &format!(
            "main:
\tmov\trax, ret_stack_rsp
\tsub\trax, 8
\tmov\tqword [ret_stack_rsp], rax
\tmov\tqword [rax], RET_MAIN
\tcall\tFN{}
RET_MAIN:
\tmov\trax, 60
\tmov\trdi, 0
\tsyscall
segment readable
",
            self.entry,
        );
        for (i, s) in self.string_literals.iter().enumerate() {
            let mut s_bytes = String::new();
            for b in s.as_bytes() {
                write!(&mut s_bytes, "{b},").unwrap();
            }
            outbuf += &format!("str_{i}: db {}\n", s_bytes.trim_end_matches(','));
        }

        outbuf
            + "segment readable writable
ret_stack_rsp: rq 1
ret_stack: rb 65536
ret_stack_end:
"
    }

    /// # Panics
    ///
    /// Will panic if trying to compile an operation that is not yet implemented.
    #[must_use]
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
                        outbuf[branch_index + 1] = (pc - branch_index) as i8 as u8;
                        vec![]
                    }
                    Op::While(_) => {
                        let branch_index = unclosed_ifs
                            .pop()
                            .expect("`end` has no opening keyword in codegen!");
                        outbuf[branch_index + 1] = (pc + 4 - branch_index) as i8 as u8;

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
            outbuf.splice(pc..pc + opcodes.len(), opcodes.iter().copied());
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
