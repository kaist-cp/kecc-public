use crate::ir::*;

use std::io::{Result, Write};

use crate::write_base::*;
use crate::*;

use lang_c::ast;

impl WriteLine for TranslationUnit {
    fn write_line(&self, indent: usize, write: &mut dyn Write) -> Result<()> {
        write_indent(indent, write)?;
        writeln!(write, "<variable list>")?;
        writeln!(write)?;
        for (name, decl) in &self.decls {
            let _ = some_or!(decl.get_variable(), continue);
            (name, decl).write_line(indent, write)?;
        }

        writeln!(write)?;
        writeln!(write)?;
        write_indent(indent, write)?;
        writeln!(write, "<function list>")?;
        writeln!(write)?;
        for (name, decl) in &self.decls {
            let _ = some_or!(decl.get_function(), continue);
            (name, decl).write_line(indent, write)?;
        }

        Ok(())
    }
}

impl WriteLine for (&String, &Declaration) {
    fn write_line(&self, indent: usize, write: &mut dyn Write) -> Result<()> {
        let name = self.0;
        let decl = self.1;

        match decl {
            Declaration::Variable { dtype, .. } => {
                writeln!(write, "{} = {}", name, dtype)?;
            }
            Declaration::Function {
                signature,
                definition,
            } => {
                let declaration = format!(
                    "{} @{}({})",
                    signature.ret,
                    name,
                    signature
                        .params
                        .iter()
                        .map(|d| d.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                );

                match definition.as_ref() {
                    Some(defintion) => {
                        // print meta data for function
                        writeln!(
                            write,
                            "; function meta data:\n;   bid_init: {}\n;   allocations: {}",
                            defintion.bid_init,
                            defintion
                                .allocations
                                .iter()
                                .enumerate()
                                .map(|(i, a)| format!("{}:{}", i, a))
                                .collect::<Vec<_>>()
                                .join(", ")
                        )?;

                        // print function definition
                        writeln!(write, "define {} {{", declaration)?;

                        for (id, block) in &defintion.blocks {
                            writeln!(write, "; <BoxId> {}", id)?;
                            (id, block).write_line(indent + 1, write)?;
                            writeln!(write)?;
                        }

                        writeln!(write, "}}")?;
                        writeln!(write)?;
                    }
                    None => {
                        // print declaration line only
                        writeln!(write, "declare {}", declaration)?;
                        writeln!(write)?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl WriteLine for (&BlockId, &Block) {
    fn write_line(&self, indent: usize, write: &mut dyn Write) -> Result<()> {
        for (i, instr) in self.1.instructions.iter().enumerate() {
            write_indent(indent, write)?;
            writeln!(
                write,
                "{}:{} = {}",
                RegisterId::temp(self.0.clone(), i),
                instr.dtype(),
                instr.write_string()
            )?;
        }

        write_indent(indent, write)?;
        writeln!(write, "{}", self.1.exit.write_string())?;

        Ok(())
    }
}

impl WriteString for Instruction {
    fn write_string(&self) -> String {
        match self {
            Instruction::BinOp { op, lhs, rhs, .. } => format!(
                "{} {} {}",
                op.write_operation(),
                lhs.write_string(),
                rhs.write_string()
            ),
            Instruction::UnaryOp { op, operand, .. } => {
                format!("{} {}", op.write_operation(), operand.write_string(),)
            }
            Instruction::Store { ptr, value } => {
                format!("store {} {}", value.write_string(), ptr.write_string())
            }
            Instruction::Load { ptr } => format!("load {}", ptr.write_string()),
            Instruction::Call { callee, args, .. } => format!(
                "call {}({})",
                callee,
                args.iter()
                    .map(WriteString::write_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Instruction::TypeCast {
                value,
                target_dtype,
            } => format!("typecast {} to {}", value.write_string(), target_dtype),
        }
    }
}

impl WriteString for Operand {
    fn write_string(&self) -> String {
        format!("{}:{}", self, self.dtype())
    }
}

impl WriteOp for ast::BinaryOperator {
    fn write_operation(&self) -> String {
        match self {
            Self::Multiply => "mul",
            Self::Divide => "div",
            Self::Modulo => "mod",
            Self::Plus => "add",
            Self::Minus => "sub",
            Self::Equals => "cmp eq",
            Self::NotEquals => "cmp ne",
            Self::Less => "cmp lt",
            Self::LessOrEqual => "cmp le",
            Self::Greater => "cmp gt",
            Self::GreaterOrEqual => "cmp ge",
            _ => todo!(),
        }
        .to_string()
    }
}

impl WriteOp for ast::UnaryOperator {
    fn write_operation(&self) -> String {
        match self {
            Self::Minus => "minus",
            _ => todo!(),
        }
        .to_string()
    }
}

impl WriteString for BlockExit {
    fn write_string(&self) -> String {
        match self {
            BlockExit::Jump { bid } => format!("j {}", bid),
            BlockExit::ConditionalJump {
                condition,
                bid_then,
                bid_else,
            } => format!(
                "br {}, {}, {}",
                condition.write_string(),
                bid_then,
                bid_else
            ),
            BlockExit::Switch {
                value,
                default,
                cases,
            } => format!(
                "switch {}, default: {} [\n{}\n  ]",
                value.write_string(),
                default,
                cases
                    .iter()
                    .map(|(v, b)| format!("    {}:{}, {}", v, v.dtype(), b))
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
            BlockExit::Return { value } => format!("ret {}", value.write_string()),
            BlockExit::Unreachable => "<unreachable>\t\t\t\t; error state".to_string(),
        }
    }
}

pub fn write_ir(ir: &TranslationUnit, write: &mut dyn Write) -> Result<()> {
    ir.write_line(0, write)
}
