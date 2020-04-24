use crate::ir::*;

use std::io::{Result, Write};

use crate::write_base::*;
use crate::*;

use lang_c::ast;

impl WriteLine for TranslationUnit {
    fn write_line(&self, indent: usize, write: &mut dyn Write) -> Result<()> {
        // TODO: consider KECC IR parser in the future.
        for (name, struct_type) in &self.structs {
            let definition = if let Some(struct_type) = struct_type {
                let fields = struct_type
                    .get_struct_fields()
                    .expect("`struct_type` must be struct type")
                    .as_ref()
                    .expect("`fields` must be `Some`");

                let fields = fields
                    .iter()
                    .map(|f| f.deref().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

                format!("{{ {} }}", fields)
            } else {
                "opaque".to_string()
            };

            writeln!(write, "struct {} : {}", name, definition)?;
        }

        for (name, decl) in &self.decls {
            let _ = some_or!(decl.get_variable(), continue);
            (name, decl).write_line(indent, write)?;
        }

        for (name, decl) in &self.decls {
            let _ = some_or!(decl.get_function(), continue);
            writeln!(write)?;
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
            Declaration::Variable { dtype, initializer } => {
                writeln!(
                    write,
                    "var {} @{} = {}",
                    dtype,
                    name,
                    if let Some(init) = initializer {
                        init.write_string()
                    } else {
                        "default".to_string()
                    }
                )?;
            }
            Declaration::Function {
                signature,
                definition,
            } => {
                if let Some(definition) = definition.as_ref() {
                    // print function definition
                    writeln!(write, "fun {} @{} {{", signature.ret, name)?;
                    // print meta data for function
                    writeln!(
                        write,
                        "init:\n  bid: {}\n  allocations: \n{}",
                        definition.bid_init,
                        definition
                            .allocations
                            .iter()
                            .enumerate()
                            .map(|(i, a)| format!(
                                "    %l{}:{}{}",
                                i,
                                a.deref(),
                                if let Some(name) = a.name() {
                                    format!(":{}", name)
                                } else {
                                    "".into()
                                }
                            ))
                            .collect::<Vec<_>>()
                            .join("\n")
                    )?;

                    for (id, block) in &definition.blocks {
                        writeln!(write, "\nblock {}:", id)?;
                        (id, block).write_line(indent + 1, write)?;
                    }

                    writeln!(write, "}}")?;
                } else {
                    // print declaration line only
                    writeln!(write, "fun {} @{}", signature.ret, name)?;
                    writeln!(write)?;
                }
            }
        }

        Ok(())
    }
}

impl WriteLine for (&BlockId, &Block) {
    fn write_line(&self, indent: usize, write: &mut dyn Write) -> Result<()> {
        for (i, phi) in self.1.phinodes.iter().enumerate() {
            write_indent(indent, write)?;
            writeln!(
                write,
                "{}:{}{}",
                RegisterId::arg(*self.0, i),
                phi.deref(),
                if let Some(name) = phi.name() {
                    format!(":{}", name)
                } else {
                    "".into()
                }
            )?;
        }

        for (i, instr) in self.1.instructions.iter().enumerate() {
            write_indent(indent, write)?;
            writeln!(
                write,
                "{}:{}{} = {}",
                RegisterId::temp(*self.0, i),
                instr.dtype(),
                if let Some(name) = instr.name() {
                    format!(":{}", name)
                } else {
                    "".into()
                },
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
            Instruction::Nop => "nop".into(),
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
            Instruction::GetElementPtr { ptr, offset, .. } => {
                format!("getelementptr {} offset {}", ptr, offset)
            }
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
        // TODO: represent signed & unsigned if necessary
        match self {
            Self::Multiply => "mul",
            Self::Divide => "div",
            Self::Modulo => "mod",
            Self::Plus => "add",
            Self::Minus => "sub",
            Self::ShiftLeft => "shl",
            Self::ShiftRight => "shr",
            Self::Equals => "cmp eq",
            Self::NotEquals => "cmp ne",
            Self::Less => "cmp lt",
            Self::LessOrEqual => "cmp le",
            Self::Greater => "cmp gt",
            Self::GreaterOrEqual => "cmp ge",
            Self::BitwiseAnd => "and",
            Self::BitwiseXor => "xor",
            Self::BitwiseOr => "or",
            _ => todo!(
                "ast::BinaryOperator::WriteOp: write operation for {:?} is needed",
                self
            ),
        }
        .to_string()
    }
}

impl WriteOp for ast::UnaryOperator {
    fn write_operation(&self) -> String {
        match self {
            Self::Plus => "plus",
            Self::Minus => "minus",
            Self::Negate => "negate",
            _ => todo!(
                "ast::UnaryOperator::WriteOp: write operation for {:?} is needed",
                self
            ),
        }
        .to_string()
    }
}

impl WriteString for BlockExit {
    fn write_string(&self) -> String {
        match self {
            BlockExit::Jump { arg } => format!("j {}", arg),
            BlockExit::ConditionalJump {
                condition,
                arg_then,
                arg_else,
            } => format!(
                "br {}, {}, {}",
                condition.write_string(),
                arg_then,
                arg_else
            ),
            BlockExit::Switch {
                value,
                default,
                cases,
            } => format!(
                "switch {} default {} [\n{}\n  ]",
                value.write_string(),
                default,
                cases
                    .iter()
                    .map(|(v, b)| format!("    {}:{} {}", v, v.dtype(), b))
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
            BlockExit::Return { value } => format!("ret {}", value.write_string()),
            BlockExit::Unreachable => "<unreachable>\t\t\t\t; error state".to_string(),
        }
    }
}
