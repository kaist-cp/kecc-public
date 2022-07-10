use crate::ir::*;

use std::io::{Result, Write};

use crate::write_base::*;
use crate::*;

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

                let fields = fields.iter().format_with(", ", |field, f| {
                    f(&format_args!(
                        "{}:{}",
                        if let Some(name) = field.name() {
                            name
                        } else {
                            "%anon"
                        },
                        field.deref()
                    ))
                });

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
                let params = signature.params.iter().format(", ");

                if let Some(definition) = definition.as_ref() {
                    // print function definition
                    writeln!(write, "fun {} @{} ({}) {{", signature.ret, name, params)?;
                    // print meta data for function
                    writeln!(
                        write,
                        "init:\n  bid: {}\n  allocations: \n{}",
                        definition.bid_init,
                        definition
                            .allocations
                            .iter()
                            .enumerate()
                            .format_with("\n", |(i, a), f| f(&format_args!(
                                "    %l{}:{}{}",
                                i,
                                a.deref(),
                                if let Some(name) = a.name() {
                                    format!(":{}", name)
                                } else {
                                    "".into()
                                }
                            )))
                    )?;

                    for (id, block) in &definition.blocks {
                        writeln!(write, "\nblock {}:", id)?;
                        (id, block).write_line(indent + 1, write)?;
                    }

                    writeln!(write, "}}")?;
                } else {
                    // print declaration line only
                    writeln!(write, "fun {} @{} ({})", signature.ret, name, params)?;
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
        format!("{}", self)
    }
}

impl WriteString for Operand {
    fn write_string(&self) -> String {
        format!("{}:{}", self, self.dtype())
    }
}

impl WriteString for BlockExit {
    fn write_string(&self) -> String {
        format!("{}", self)
    }
}
