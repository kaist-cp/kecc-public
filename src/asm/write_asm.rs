use std::io::{Result, Write};

use crate::asm::*;
use crate::write_base::*;

const INDENT: usize = 4;

impl WriteLine for Asm {
    fn write_line(&self, indent: usize, write: &mut dyn Write) -> Result<()> {
        self.unit.write_line(indent, write)
    }
}

impl WriteLine for TranslationUnit {
    fn write_line(&self, indent: usize, write: &mut dyn Write) -> Result<()> {
        for function in &self.functions {
            function.write_line(indent, write)?;
        }

        for variable in &self.variables {
            variable.write_line(indent, write)?;
        }

        Ok(())
    }
}

impl<T: WriteLine> WriteLine for Section<T> {
    fn write_line(&self, indent: usize, write: &mut dyn Write) -> Result<()> {
        for directive in &self.header {
            write_indent(indent + INDENT, write)?;
            writeln!(write, "{}", directive.write_string())?;
        }
        self.body.write_line(indent, write)?;

        Ok(())
    }
}

impl WriteLine for Function {
    fn write_line(&self, indent: usize, write: &mut dyn Write) -> Result<()> {
        for block in &self.blocks {
            block.write_line(indent, write)?;
        }

        Ok(())
    }
}

impl WriteLine for Variable {
    fn write_line(&self, _indent: usize, _write: &mut dyn Write) -> Result<()> {
        todo!()
    }
}

impl WriteLine for Block {
    fn write_line(&self, indent: usize, write: &mut dyn Write) -> Result<()> {
        if let Some(label) = &self.label {
            writeln!(write, "{}:", label.0)?;
        }

        for instruction in &self.instructions {
            write_indent(indent + INDENT, write)?;
            writeln!(write, "{}", instruction.write_string())?;
        }

        Ok(())
    }
}

impl WriteString for Directive {
    fn write_string(&self) -> String {
        match self {
            Self::Globl(label) => format!(".globl\t{}", label.0),
            Self::Type(symbol, symbol_type) => {
                format!(".type\t{}, {}", symbol.0, symbol_type.write_string())
            }
        }
    }
}

impl WriteString for SymbolType {
    fn write_string(&self) -> String {
        match self {
            Self::Function => "@function",
            Self::Object => "@object",
        }
        .to_string()
    }
}

impl WriteString for Instruction {
    fn write_string(&self) -> String {
        match self {
            Self::RType {
                instr,
                rd,
                rs1,
                rs2,
            } => format!(
                "{}\t{},{},{}",
                instr.write_string(),
                rd.write_string(),
                rs1.write_string(),
                rs2.write_string()
            ),
            Self::IType {
                instr,
                rd,
                rs1,
                imm,
            } => {
                if let IType::Load(_) = instr {
                    format!(
                        "{}\t{},{}({})",
                        instr.write_string(),
                        rd.write_string(),
                        imm,
                        rs1.write_string()
                    )
                } else {
                    format!(
                        "{}\t{},{},{}",
                        instr.write_string(),
                        rd.write_string(),
                        rs1.write_string(),
                        imm
                    )
                }
            }
            Self::SType {
                instr,
                rs1,
                rs2,
                imm,
            } => format!(
                "{}\t{},{}({})",
                instr.write_string(),
                rs2.write_string(),
                imm.to_string(),
                rs1.write_string()
            ),
            Self::BType {
                instr,
                rs1,
                rs2,
                imm,
            } => format!(
                "{}\t{},{}, {}",
                instr.write_string(),
                rs1.write_string(),
                rs2.write_string(),
                imm.0,
            ),
            Self::Pseudo(pseudo) => pseudo.write_string(),
        }
    }
}

impl WriteString for RType {
    fn write_string(&self) -> String {
        match self {
            Self::Add(data_size) => format!("add{}", data_size.write_string()),
            Self::Sub(data_size) => format!("sub{}", data_size.write_string()),
            Self::Mul(data_size) => format!("mul{}", data_size.write_string()),
            Self::Div(data_size, is_signed) => format!(
                "div{}{}",
                if *is_signed { "" } else { "u" },
                data_size.write_string()
            ),
            Self::Slt(is_signed) => format!("slt{}", if *is_signed { "" } else { "u" }),
            Self::Xor => "xor".to_string(),
        }
    }
}

impl WriteString for IType {
    fn write_string(&self) -> String {
        match self {
            Self::Load(data_size) => format!("l{}", data_size.write_string()),
            Self::Addi(data_size) => format!("addi{}", data_size.write_string()),
            Self::Andi => "andi".to_string(),
            Self::Slli => "slli".to_string(),
            Self::Srli => "srli".to_string(),
        }
    }
}

impl WriteString for SType {
    fn write_string(&self) -> String {
        match self {
            Self::Store(data_size) => format!("s{}", data_size.write_string()),
        }
    }
}

impl WriteString for BType {
    fn write_string(&self) -> String {
        match self {
            Self::Beq => "beq".to_string(),
            Self::Bne => "bne".to_string(),
            Self::Blt(is_signed) => format!("blt{}", if *is_signed { "" } else { "u" }),
            Self::Bge(is_signed) => format!("bge{}", if *is_signed { "" } else { "u" }),
        }
    }
}

impl WriteString for Pseudo {
    fn write_string(&self) -> String {
        match self {
            Self::Li { rd, imm } => format!("li\t{},{}", rd.write_string(), imm),
            Self::Mv { rs, rd } => format!("mv\t{},{}", rd.write_string(), rs.write_string()),
            Self::Neg { data_size, rs, rd } => format!(
                "neg{}\t{},{}",
                data_size.write_string(),
                rd.write_string(),
                rs.write_string()
            ),
            Self::SextW { rs, rd } => {
                format!("sext.w\t{},{}", rd.write_string(), rs.write_string())
            }
            Self::Seqz { rs, rd } => format!("seqz\t{},{}", rd.write_string(), rs.write_string()),
            Self::J { offset } => format!("j\t{}", offset.0),
            Self::Jr { rs } => format!("jr\t{}", rs.write_string()),
            Self::Ret => "ret".to_string(),
            Self::Call { offset } => format!("call\t{}", offset.0),
        }
    }
}

impl WriteString for DataSize {
    fn write_string(&self) -> String {
        match self {
            Self::Byte => "b",
            Self::Half => "h",
            Self::Word => "w",
            Self::Double => "d",
        }
        .to_string()
    }
}

impl WriteString for Register {
    fn write_string(&self) -> String {
        match self {
            Self::Zero => "zero".to_string(),
            Self::Ra => "ra".to_string(),
            Self::Sp => "sp".to_string(),
            Self::Gp => "gp".to_string(),
            Self::Tp => "tp".to_string(),
            Self::Temp(id) => format!("t{}", id),
            Self::Saved(id) => format!("s{}", id),
            Self::Arg(id) => format!("a{}", id),
        }
    }
}
