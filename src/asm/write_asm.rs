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
    fn write_line(&self, indent: usize, write: &mut dyn Write) -> Result<()> {
        writeln!(write, "{}:", self.label.0)?;
        for directive in &self.directives {
            write_indent(indent + INDENT, write)?;
            writeln!(write, "{}", directive.write_string())?;
        }

        Ok(())
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
            Self::Align(value) => format!(".align\t{}", value),
            Self::Globl(label) => format!(".globl\t{}", label.0),
            Self::Type(symbol, symbol_type) => {
                format!(".type\t{}, {}", symbol.0, symbol_type.write_string())
            }
            Self::Section(section_type) => format!(".section\t{}", section_type.write_string()),
            Self::Byte(value) => format!(".byte\t{:#x?}", value),
            Self::Half(value) => format!(".half\t{:#x?}", value),
            Self::Word(value) => format!(".word\t{:#x?}", value),
            Self::Quad(value) => format!(".quad\t{:#x?}", value),
        }
    }
}

impl WriteString for SectionType {
    fn write_string(&self) -> String {
        match self {
            Self::Text => ".text",
            Self::Data => ".data",
            Self::Rodata => ".rodata",
            Self::Bss => ".bss",
        }
        .to_string()
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
            } => {
                let rounding_mode = if let RType::FcvtFloatToInt { .. } = instr {
                    ",rtz"
                } else {
                    ""
                }
                .to_string();

                format!(
                    "{}\t{},{}{}{}",
                    instr.write_string(),
                    rd.write_string(),
                    rs1.write_string(),
                    if let Some(rs2) = rs2 {
                        format!(",{}", rs2.write_string())
                    } else {
                        "".to_string()
                    },
                    rounding_mode
                )
            }
            Self::IType {
                instr,
                rd,
                rs1,
                imm,
            } => {
                if let IType::Load { .. } = instr {
                    format!(
                        "{}\t{},{}({})",
                        instr.write_string(),
                        rd.write_string(),
                        imm.write_string(),
                        rs1.write_string()
                    )
                } else {
                    format!(
                        "{}\t{},{},{}",
                        instr.write_string(),
                        rd.write_string(),
                        rs1.write_string(),
                        imm.write_string(),
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
                imm.write_string(),
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
            Self::UType { instr, rd, imm } => format!(
                "{}\t{}, {}",
                instr.write_string(),
                rd.write_string(),
                imm.write_string(),
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
            Self::Sll(data_size) => format!("sll{}", data_size.write_string()),
            Self::Srl(data_size) => format!("srl{}", data_size.write_string()),
            Self::Sra(data_size) => format!("sra{}", data_size.write_string()),
            Self::Mul(data_size) => format!("mul{}", data_size.write_string()),
            Self::Div {
                data_size,
                is_signed,
            } => format!(
                "div{}{}",
                if *is_signed { "" } else { "u" },
                data_size.write_string()
            ),
            Self::Rem {
                data_size,
                is_signed,
            } => format!(
                "rem{}{}",
                if *is_signed { "" } else { "u" },
                data_size.write_string()
            ),
            Self::Slt { is_signed } => format!("slt{}", if *is_signed { "" } else { "u" }),
            Self::Xor => "xor".to_string(),
            Self::Or => "or".to_string(),
            Self::And => "and".to_string(),
            Self::Fadd(data_size) => format!("fadd.{}", data_size.write_string()),
            Self::Fsub(data_size) => format!("fsub.{}", data_size.write_string()),
            Self::Fmul(data_size) => format!("fmul.{}", data_size.write_string()),
            Self::Fdiv(data_size) => format!("fdiv.{}", data_size.write_string()),
            Self::Feq(data_size) => format!("feq.{}", data_size.write_string()),
            Self::Flt(data_size) => format!("flt.{}", data_size.write_string()),
            Self::FmvFloatToInt { float_data_size } => {
                assert!(float_data_size.is_floating_point());
                format!(
                    "fmv.x.{}",
                    if *float_data_size == DataSize::SinglePrecision {
                        "w"
                    } else {
                        "d"
                    }
                )
            }
            Self::FmvIntToFloat { float_data_size } => {
                assert!(float_data_size.is_floating_point());
                format!(
                    "fmv.{}.x",
                    if *float_data_size == DataSize::SinglePrecision {
                        "w"
                    } else {
                        "d"
                    }
                )
            }
            Self::FcvtFloatToInt {
                float_data_size,
                int_data_size,
                is_signed,
            } => {
                assert!(float_data_size.is_floating_point());
                format!(
                    "fcvt.{}{}.{}",
                    if let Some(int_data_size) = int_data_size {
                        assert_eq!(*int_data_size, DataSize::Word);
                        "w"
                    } else {
                        "l"
                    }
                    .to_string(),
                    if *is_signed { "" } else { "u" },
                    float_data_size.write_string()
                )
            }
            Self::FcvtIntToFloat {
                int_data_size,
                float_data_size,
                is_signed,
            } => {
                assert!(float_data_size.is_floating_point());
                format!(
                    "fcvt.{}.{}{}",
                    float_data_size.write_string(),
                    if let Some(int_data_size) = int_data_size {
                        assert_eq!(*int_data_size, DataSize::Word);
                        "w"
                    } else {
                        "l"
                    }
                    .to_string(),
                    if *is_signed { "" } else { "u" }
                )
            }
            Self::FcvtFloatToFloat { from, to } => {
                assert!(from.is_floating_point());
                assert!(to.is_floating_point());
                format!("fcvt.{}.{}", to.write_string(), from.write_string())
            }
        }
    }
}

impl WriteString for IType {
    fn write_string(&self) -> String {
        match self {
            Self::Load {
                data_size,
                is_signed,
            } => {
                if data_size.is_integer() {
                    format!(
                        "l{}{}",
                        data_size.write_string(),
                        if *is_signed { "" } else { "u" }
                    )
                } else {
                    format!(
                        "fl{}",
                        if *data_size == DataSize::SinglePrecision {
                            "w"
                        } else {
                            "d"
                        }
                    )
                }
            }
            Self::Addi(data_size) => format!("addi{}", data_size.write_string()),
            Self::Xori => "xori".to_string(),
            Self::Ori => "ori".to_string(),
            Self::Andi => "andi".to_string(),
            Self::Slli(data_size) => format!("slli{}", data_size.write_string()),
            Self::Srli(data_size) => format!("srli{}", data_size.write_string()),
            Self::Srai(data_size) => format!("srai{}", data_size.write_string()),
            Self::Slti { is_signed } => format!("slti{}", if *is_signed { "" } else { "u" }),
        }
    }
}

impl WriteString for SType {
    fn write_string(&self) -> String {
        match self {
            Self::Store(data_size) => {
                if data_size.is_integer() {
                    format!("s{}", data_size.write_string())
                } else {
                    format!(
                        "fs{}",
                        if *data_size == DataSize::SinglePrecision {
                            "w"
                        } else {
                            "d"
                        }
                    )
                }
            }
        }
    }
}

impl WriteString for BType {
    fn write_string(&self) -> String {
        match self {
            Self::Beq => "beq".to_string(),
            Self::Bne => "bne".to_string(),
            Self::Blt { is_signed } => format!("blt{}", if *is_signed { "" } else { "u" }),
            Self::Bge { is_signed } => format!("bge{}", if *is_signed { "" } else { "u" }),
        }
    }
}

impl WriteString for UType {
    fn write_string(&self) -> String {
        match self {
            Self::Lui => "lui".to_string(),
        }
    }
}

impl WriteString for Pseudo {
    fn write_string(&self) -> String {
        match self {
            Self::Li { rd, imm } => format!("li\t{},{}", rd.write_string(), *imm as i64),
            Self::Mv { rd, rs } => format!("mv\t{},{}", rd.write_string(), rs.write_string()),
            Self::Fmv { data_size, rd, rs } => format!(
                "fmv.{}\t{},{}",
                data_size.write_string(),
                rd.write_string(),
                rs.write_string()
            ),
            Self::Neg { data_size, rd, rs } => format!(
                "neg{}\t{},{}",
                data_size.write_string(),
                rd.write_string(),
                rs.write_string()
            ),
            Self::SextW { rs, rd } => {
                format!("sext.w\t{},{}", rd.write_string(), rs.write_string())
            }
            Self::Seqz { rd, rs } => format!("seqz\t{},{}", rd.write_string(), rs.write_string()),
            Self::Snez { rd, rs } => format!("snez\t{},{}", rd.write_string(), rs.write_string()),
            Self::Fneg { data_size, rd, rs } => format!(
                "fneg.{}\t{},{}",
                data_size.write_string(),
                rd.write_string(),
                rs.write_string()
            ),
            Self::J { offset } => format!("j\t{}", offset.0),
            Self::Jr { rs } => format!("jr\t{}", rs.write_string()),
            Self::Jalr { rs } => format!("jalr\t{}", rs.write_string()),
            Self::Ret => "ret".to_string(),
            Self::Call { offset } => format!("call\t{}", offset.0),
        }
    }
}

impl WriteString for Immediate {
    fn write_string(&self) -> String {
        match self {
            Self::Value(value) => format!("{}", *value as i64),
            Self::Relocation { relocation, symbol } => {
                format!("{}({})", relocation.write_string(), symbol.0)
            }
        }
    }
}

impl WriteString for RelocationFunction {
    fn write_string(&self) -> String {
        match self {
            Self::HI20 => "%hi",
            Self::LO12 => "%lo",
        }
        .to_string()
    }
}

impl WriteString for DataSize {
    fn write_string(&self) -> String {
        match self {
            Self::Byte => "b",
            Self::Half => "h",
            Self::Word => "w",
            Self::Double => "d",
            Self::SinglePrecision => "s",
            Self::DoublePrecision => "d",
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
            Self::Temp(registr_type, id) => format!(
                "{}t{}",
                if *registr_type == RegisterType::FloatingPoint {
                    "f"
                } else {
                    ""
                },
                id
            ),
            Self::Saved(registr_type, id) => format!(
                "{}s{}",
                if *registr_type == RegisterType::FloatingPoint {
                    "f"
                } else {
                    ""
                },
                id
            ),
            Self::Arg(registr_type, id) => format!(
                "{}a{}",
                if *registr_type == RegisterType::FloatingPoint {
                    "f"
                } else {
                    ""
                },
                id
            ),
        }
    }
}
