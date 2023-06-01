use std::fmt::Write;

use crate::{Func, Op, Program};

pub mod fasm_x86_64_linux {
    use super::*;

    fn write_op(op: &Op, count_ops: &mut usize, buffer: &mut String) -> std::fmt::Result {
        match op {
            Op::CallFn(index) => write!(
                buffer,
                "\tmov\trax, [ret_stack_rsp]
\tsub\trax, 8
\tmov\t[ret_stack_rsp], rax
\tmov\tqword [rax], RET{count_ops}
\tjmp\tFN{index}
RET{count_ops}:
\tmov\trax, [ret_stack_rsp]
\tadd\trax, 8
\tmov\t[ret_stack_rsp], rax
"
            )?,
            // Small optimisation for the 0 case.
            Op::Ret(0) => write!(
                buffer,
                "\tmov\trax, qword [ret_stack_rsp]
\tjmp qword [rax]
"
            )?,
            Op::Ret(count) => write!(
                buffer,
                "\tmov\trax, [ret_stack_rsp]
\tadd\trax, {}
\tmov\tqword [ret_stack_rsp], rax
\tjmp\tqword [rax]
",
                count * 8
            )?,
            Op::Bind { count, peek, body } => {
                write!(
                    buffer,
                    "\tmov\trax, [ret_stack_rsp]
\tsub\trax, {}
\tmov\t[ret_stack_rsp], rax
",
                    count * 8
                )?;
                for i in 0..*count {
                    write!(
                        buffer,
                        "\tmov\trbx, [rsp + {0}]
\tmov\t[rax+{0}], rbx
",
                        i * 8
                    );
                }
                if !peek {
                    write!(buffer, "\tadd\trsp, {}\n", count * 8)?;
                }
                for op in body {
                    write_op(op, count_ops, buffer)?;
                }
                // Remove the bindings from the return stack.
                write!(
                    buffer,
                    "\tmov\trax, [ret_stack_rsp]
\tadd\trax, {}
\tmov\tqword [ret_stack_rsp], rax
",
                    count * 8
                )?;
            }
            Op::PushBind(index) => write!(
                buffer,
                "\tmov\trax, [ret_stack_rsp]\t; Op::PushBind({index})
\tadd\trax, {}
\tpush\tqword [rax]
",
                index * 8
            )?,
            Op::PushInt(val) => write!(buffer, "\tpush\t{0}\t\t\t; Op::PushInt({0})\n", val)?,
            Op::PushStrPtr(index) => write!(
                buffer,
                "\tpush\tstr_{0}\t\t\t; Op::PushStrPtr({0})\n",
                index
            )?,
            Op::Plus => write!(
                buffer,
                "\tpop\trax\t\t\t; Op::Plus
\tpop\trbx
\tadd\trax, rbx
\tpush\trax
",
            )?,
            Op::Minus => write!(
                buffer,
                "\tpop\trbx\t\t\t; Op::Minus
\tpop\trax
\tsub\trax, rbx
\tpush\trax
",
            )?,
            Op::DivMod => write!(
                buffer,
                "\tpop\trbx\t\t\t; Op::DivMod
\tpop\trax
\tmov\trdx, 0
\tdiv\trbx
\tpush\trax
\tpush\trdx
",
            )?,
            Op::Dup => write!(buffer, "\tpush\tqword [rsp]\t\t; Op::Dup",)?,
            Op::Drop => write!(buffer, "\tadd\trsp, 8\t\t\t; Op::Drop",)?,
            Op::Swap => write!(
                buffer,
                "\tpop\trax\t\t\t; Op::Swap
\tpop\trbx
\tpush\trax
\tpush\trbx
",
            )?,
            Op::Over => write!(
                buffer,
                "\tpop\trax\t\t\t; Op::Over
\tpop\trbx
\tpop\trcx
\tpush\trbx
\tpush\trax
\tpush\trcx
"
            )?,
            Op::Equals => {
                write!(
                    buffer,
                    "\tpop\trax\t\t\t; Op::Equals
\tpop\trbx
\tcmp\trax, rbx
\tje\tJ{0}
\tpush\t0
\tjmp\tJ{1}
J{0}:
\tpush\t1
J{1}:
",
                    count_ops,
                    *count_ops + 1,
                )?;
                *count_ops += 1;
            }
            Op::Neq => {
                write!(
                    buffer,
                    "\tpop\trax\t\t\t; Op::Neq
\tpop\trbx
\tcmp\trax, rbx
\tjne\tJ{0}
\tpush\t0
\tjmp\tJ{1}
J{0}:
\tpush\t1
J{1}:
",
                    count_ops,
                    *count_ops + 1,
                )?;
                *count_ops += 1;
            }
            Op::Not => write!(
                buffer,
                "\tpop\trax\t\t\t; Op::Not
\tmov\trbx, 1
\tsub\trbx, rax
\tpush\trbx
"
            )?,
            Op::GreaterThan => {
                write!(
                    buffer,
                    "\tpop\trax\t\t\t; Op::GreaterThan
\tpop\trbx
\tcmp\trax, rbx
\tjb\tJ{0}
\tpush\t0
\tjmp\tJ{1}
J{0}:
\tpush\t1
J{1}:
",
                    count_ops,
                    *count_ops + 1,
                )?;
                *count_ops += 1;
            }
            Op::LessThan => {
                write!(
                    buffer,
                    "\tpop\trax\t\t\t; Op::LessThan
\tpop\trbx
\tcmp\trbx, rax
\tjb\tJ{0}
\tpush\t0
\tjmp\tJ{1}
J{0}:
\tpush\t1
J{1}:
",
                    count_ops,
                    *count_ops + 1,
                )?;
                *count_ops += 1;
            }
            Op::Or => {
                write!(
                    buffer,
                    "\tpop\trax\t\t\t; Op::Or
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
                    count_ops,
                    *count_ops + 1,
                )?;
                *count_ops += 1;
            }
            Op::And => {
                write!(
                    buffer,
                    "\tpop\trax\t\t\t; Op::And
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
                    count_ops,
                    *count_ops + 1,
                )?;
                *count_ops += 1;
            }
            Op::ReadByte => write!(
                buffer,
                "\tpop\trbx\t\t\t; Op::ReadByte
\tmov\trax, 0
\tmov\tal, byte [rbx]
\tpush\trax
"
            )?,
            Op::If(ops) => {
                let jump_to = *count_ops;
                *count_ops += 1;
                write!(
                    buffer,
                    "\tpop\trax\t\t\t; Op::If
\tcmp\trax, 1
\tjne\tF{jump_to}
"
                )?;
                for op in ops {
                    write_op(op, count_ops, buffer)?;
                }
                write!(buffer, "F{jump_to}")?;
            }
            Op::While { condn, body } => {
                let condn_jump = *count_ops;
                let end_jump = *count_ops + 1;
                *count_ops += 2;
                write!(buffer, "F{condn_jump}:\t\t\t\t\t; Op::While")?;
                for op in condn {
                    write_op(op, count_ops, buffer)?;
                }
                // Check the while condition and jump to end if not met.
                write!(
                    buffer,
                    "\tpop\trax
\tcmp\trax, 1
\tjne\tF{end_jump}
"
                )?;
                for op in body {
                    write_op(op, count_ops, buffer)?;
                }
                write!(buffer, "F{end_jump}:")?;
            }
            Op::Print => write!(buffer, "\tpop\trdi\t\t\t; Op::Print\n\tcall\tprint\n")?,
            Op::Puts => write!(
                buffer,
                "\tmov\trdi, 1\t\t\t; Op::Puts
\tpop\trsi
\tpop\trdx
\tmov\trax, 1
\tsyscall
"
            )?,
        }
        *count_ops += 1;
        Ok(())
    }

    #[must_use]
    pub fn generate(program: Program) -> Result<String, std::fmt::Error> {
        let mut outbuf = String::from(
            "format ELF64 executable 3
entry main
segment readable executable
",
        );

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

        let mut count_ops = 0;

        for func in program.funcs.iter() {
            write!(outbuf, "fn_{}:", func.ident)?;
            for op in &func.body {
                write_op(op, &mut count_ops, &mut outbuf);
            }
        }

        write!(
            outbuf,
            "main:
\tmov\trax, ret_stack_rsp
\tsub\trax, 8
\tmov\tqword [ret_stack_rsp], rax
\tmov\tqword [rax], RET_MAIN
\tcall\tfn_main
RET_MAIN:
\tmov\trax, 60
\tmov\trdi, 0
\tsyscall
segment readable
",
        )?;
        for (i, s) in program.ctx.strings.iter().enumerate() {
            let mut s_bytes = String::new();
            for b in s.as_bytes() {
                write!(&mut s_bytes, "{b},").unwrap();
            }
            write!(outbuf, "str_{i}: db {}\n", s_bytes.trim_end_matches(','))?;
        }

        Ok(outbuf
            + "segment readable writable
ret_stack_rsp: rq 1
ret_stack: rb 65536
ret_stack_end:
")
    }
}

pub mod mos_6502_nesulator {
    use super::*;
    /// # Panics
    ///
    /// Will panic if trying to compile an operation that is not yet implemented.
    #[must_use]
    pub fn generate(program: Program) -> [u8; 65536 - 0x4020] {
        todo!()
        //const NOP: u8 = 0xea;
        //const PHA: u8 = 0x48;
        //const PLA: u8 = 0x68;
        //const CLC: u8 = 0x18;
        //const SEC: u8 = 0x38;
        //const ADC_ZPG: u8 = 0x65;
        //const SBC_ZPG: u8 = 0xe5;
        //const LDA_IMM: u8 = 0xa9;
        //const LDA_ZPG: u8 = 0xa5;
        //const STA_ZPG: u8 = 0x85;
        //const BNE: u8 = 0xd0;
        //const CMP_IMM: u8 = 0xc9;
        //const CMP_ZPG: u8 = 0xc5;

        //let mut outbuf = vec![NOP; 65536 - 0x4020];
        //outbuf[0xfffc - 0x4020] = 0x20;
        //outbuf[0xfffd - 0x4020] = 0x40;

        //let mut pc: usize = 0x00;

        //let mut unclosed_ifs = Vec::new();
        //let mut unclosed_whiles = Vec::new();

        //for (loc, op) in self.program.into_iter().map(|tok| (tok.loc, tok.op)) {
        //    let opcodes = match op {
        //        Op::PushInt(val) => vec![LDA_IMM, val as u8, PHA],
        //        Op::Plus => vec![PLA, STA_ZPG, 0x00, PLA, CLC, ADC_ZPG, 0x00, PHA],
        //        Op::Minus => vec![PLA, STA_ZPG, 0x00, PLA, SEC, SBC_ZPG, 0x00, PHA],
        //        Op::Drop => vec![PLA],
        //        Op::Over => vec![
        //            PLA, STA_ZPG, 0x00, PLA, STA_ZPG, 0x01, PLA, STA_ZPG, 0x02, LDA_ZPG, 0x01, PHA,
        //            LDA_ZPG, 0x02, PHA, LDA_ZPG, 0x00, PHA,
        //        ],
        //        Op::Dup => vec![PLA, PHA, PHA],
        //        Op::Neq => vec![
        //            PLA, STA_ZPG, 0x00, PLA, CMP_ZPG, 0x00, BNE, 0x09, LDA_IMM, 0x00, PHA, LDA_IMM,
        //            0x01, BNE, 0x05, LDA_IMM, 0x01, PHA,
        //        ],
        //        Op::Swap => vec![
        //            PLA, STA_ZPG, 0x00, PLA, STA_ZPG, 0x01, LDA_ZPG, 0x00, PHA, LDA_ZPG, 0x01, PHA,
        //        ],
        //        Op::If(_) => {
        //            unclosed_ifs.push(pc + 3);
        //            vec![PLA, CMP_IMM, 0x01, BNE, 0x00]
        //        }
        //        Op::While(_) => {
        //            unclosed_whiles.push(pc);
        //            vec![]
        //        }
        //        Op::End(op) => match *op {
        //            Op::If(_) => {
        //                let branch_index = unclosed_ifs
        //                    .pop()
        //                    .expect("`end` has no opening keyword in codegen!");
        //                outbuf[branch_index + 1] = (pc - branch_index) as i8 as u8;
        //                vec![]
        //            }
        //            Op::While(_) => {
        //                let branch_index = unclosed_ifs
        //                    .pop()
        //                    .expect("`end` has no opening keyword in codegen!");
        //                outbuf[branch_index + 1] = (pc + 4 - branch_index) as i8 as u8;

        //                let while_index = unclosed_whiles
        //                    .pop()
        //                    .expect("`endwhile` has no opening `while` in codegen!");

        //                vec![
        //                    LDA_IMM,
        //                    0x01,
        //                    BNE,
        //                    (while_index.wrapping_sub(pc)) as i8 as u8,
        //                ]
        //            }
        //            _ => todo!(),
        //        },
        //        op => {
        //            eprintln!("{loc}: `{op:?}` not implemented in codegen!");
        //            process::exit(1);
        //        }
        //    };
        //    outbuf.splice(pc..pc + opcodes.len(), opcodes.iter().copied());
        //    pc += opcodes.len();
        //}

        //outbuf.try_into().unwrap_or_else(|v: Vec<u8>| {
        //    panic!(
        //        "Expected Vec of length {} but it was {}",
        //        65536 - 0x4020,
        //        v.len()
        //    )
        //})
    }
}
