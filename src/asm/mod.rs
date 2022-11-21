mod write_asm;

use crate::ir;
use crate::write_base::*;

use core::convert::TryFrom;
use core::fmt;

/// TODO
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asm {
    pub unit: TranslationUnit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationUnit {
    pub functions: Vec<Section<Function>>,
    pub variables: Vec<Section<Variable>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section<T> {
    /// Section Headers provide size, offset, type, alignment and flags of the sections
    ///
    /// For more details: <https://github.com/michaeljclark/michaeljclark.github.io/blob/master/asm.md#section-header>
    pub header: Vec<Directive>,
    pub body: T,
}

/// An object file is made up of multiple sections, with each section corresponding to distinct
/// types of executable code or data.
///
/// For more details: <https://github.com/michaeljclark/michaeljclark.github.io/blob/master/asm.md#sections>
impl<T> Section<T> {
    pub fn new(header: Vec<Directive>, body: T) -> Self {
        Self { header, body }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub blocks: Vec<Block>,
}

impl Function {
    pub fn new(blocks: Vec<Block>) -> Self {
        Self { blocks }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    pub label: Label,
    pub directives: Vec<Directive>,
}

impl Variable {
    pub fn new(label: Label, directives: Vec<Directive>) -> Self {
        Self { label, directives }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub label: Option<Label>,
    pub instructions: Vec<Instruction>,
}

impl Block {
    pub fn new(label: Option<Label>, instructions: Vec<Instruction>) -> Self {
        Self {
            label,
            instructions,
        }
    }
}

/// The assembler implements several directives that control the assembly of instructions into an
/// object file.
///
/// For more information: <https://github.com/michaeljclark/michaeljclark.github.io/blob/master/asm.md#assembler-directives>
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Directive {
    /// .align integer
    Align(usize),
    /// .globl symbol
    Globl(Label),
    /// .section section_type
    Section(SectionType),
    /// .type symbol, symbol_type
    Type(Label, SymbolType),
    /// .byte value
    Byte(u8),
    /// .half value
    Half(u16),
    /// .word value
    Word(u32),
    /// .quad value
    Quad(u64),
    /// .zero bytes
    Zero(usize),
}

impl Directive {
    pub fn try_from_data_size(data_size: DataSize, value: u64) -> Self {
        match data_size {
            DataSize::Byte => Self::Byte(value as u8),
            DataSize::Half => Self::Half(value as u16),
            DataSize::Word => Self::Word(value as u32),
            DataSize::Double => Self::Quad(value),
            DataSize::SinglePrecision => Self::Word(value as u32),
            DataSize::DoublePrecision => Self::Quad(value),
        }
    }
}

impl fmt::Display for Directive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Align(value) => write!(f, ".align\t{}", value),
            Self::Globl(label) => write!(f, ".globl\t{}", label),
            Self::Type(symbol, symbol_type) => {
                write!(f, ".type\t{}, {}", symbol, symbol_type)
            }
            Self::Section(section_type) => write!(f, ".section\t{}", section_type),
            Self::Byte(value) => write!(f, ".byte\t{:#x?}", value),
            Self::Half(value) => write!(f, ".half\t{:#x?}", value),
            Self::Word(value) => write!(f, ".word\t{:#x?}", value),
            Self::Quad(value) => write!(f, ".quad\t{:#x?}", value),
            Self::Zero(bytes) => write!(f, ".zero\t{:#x?}", bytes),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionType {
    Text,
    Data,
    Rodata,
    Bss,
}

impl fmt::Display for SectionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Text => ".text",
                Self::Data => ".data",
                Self::Rodata => ".rodata",
                Self::Bss => ".bss",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolType {
    Function,
    Object,
}

impl fmt::Display for SymbolType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Function => "@function",
                Self::Object => "@object",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    /// R-type instruction format
    ///
    /// For more details: <https://riscv.org/wp-content/uploads/2017/05/riscv-spec-v2.2.pdf> (104p)
    RType {
        instr: RType,
        rd: Register,
        rs1: Register,
        rs2: Option<Register>,
    },
    /// I-type instruction format
    ///
    /// For more details: <https://riscv.org/wp-content/uploads/2017/05/riscv-spec-v2.2.pdf> (104p)
    IType {
        instr: IType,
        rd: Register,
        rs1: Register,
        imm: Immediate,
    },
    /// S-type instruction format
    ///
    /// For more details: <https://riscv.org/wp-content/uploads/2017/05/riscv-spec-v2.2.pdf> (104p)
    SType {
        instr: SType,
        rs1: Register,
        rs2: Register,
        imm: Immediate,
    },
    /// B-type instruction format
    ///
    /// For more details: <https://riscv.org/wp-content/uploads/2017/05/riscv-spec-v2.2.pdf> (104p)
    BType {
        instr: BType,
        rs1: Register,
        rs2: Register,
        imm: Label,
    },
    UType {
        instr: UType,
        rd: Register,
        imm: Immediate,
    },
    Pseudo(Pseudo),
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

                write!(
                    f,
                    "{}\t{},{}{}{}",
                    instr,
                    rd,
                    rs1,
                    if let Some(rs2) = rs2 {
                        format!(",{}", rs2)
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
                    write!(f, "{}\t{},{}({})", instr, rd, imm, rs1)
                } else {
                    write!(f, "{}\t{},{},{}", instr, rd, rs1, imm,)
                }
            }
            Self::SType {
                instr,
                rs1,
                rs2,
                imm,
            } => write!(f, "{}\t{},{}({})", instr, rs2, imm, rs1),
            Self::BType {
                instr,
                rs1,
                rs2,
                imm,
            } => write!(f, "{}\t{},{}, {}", instr, rs1, rs2, imm.0,),
            Self::UType { instr, rd, imm } => write!(f, "{}\t{}, {}", instr, rd, imm,),
            Self::Pseudo(pseudo) => write!(f, "{}", pseudo),
        }
    }
}

/// If the enum variant contains `bool`,
/// It means that different instructions exist
/// depending on whether the operand is signed or not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RType {
    Add(DataSize),
    Sub(DataSize),
    Sll(DataSize),
    Srl(DataSize),
    Sra(DataSize),
    Mul(DataSize),
    Div {
        data_size: DataSize,
        is_signed: bool,
    },
    Rem {
        data_size: DataSize,
        is_signed: bool,
    },
    Slt {
        is_signed: bool,
    },
    Xor,
    Or,
    And,
    Fadd(DataSize),
    Fsub(DataSize),
    Fmul(DataSize),
    Fdiv(DataSize),
    Feq(DataSize),
    Flt(DataSize),
    /// fmv.w.x or fmv.d.x
    FmvIntToFloat {
        float_data_size: DataSize,
    },
    /// fmv.x.w or fmv.x.w
    FmvFloatToInt {
        float_data_size: DataSize,
    },
    /// fcvt.s.l(u) or fcvt.d.l(u)
    /// fcvt.s.w(u) or fcvt.d.w(u)
    FcvtIntToFloat {
        int_data_size: DataSize,
        float_data_size: DataSize,
        is_signed: bool,
    },
    /// fcvt.l(u).s or fcvt.l(u).d
    /// fcvt.w(u).s or fcvt.w(u).d
    FcvtFloatToInt {
        float_data_size: DataSize,
        int_data_size: DataSize,
        is_signed: bool,
    },
    /// fcvt.s.d or fcvt.d.s
    FcvtFloatToFloat {
        from: DataSize,
        to: DataSize,
    },
}

impl RType {
    pub fn add(dtype: ir::Dtype) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(matches!(data_size, DataSize::Word | DataSize::Double));

        Self::Add(data_size)
    }

    pub fn sub(dtype: ir::Dtype) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(matches!(data_size, DataSize::Word | DataSize::Double));

        Self::Sub(data_size)
    }

    pub fn sll(dtype: ir::Dtype) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(matches!(data_size, DataSize::Word | DataSize::Double));

        Self::Sll(data_size)
    }

    pub fn srl(dtype: ir::Dtype) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(matches!(data_size, DataSize::Word | DataSize::Double));

        Self::Srl(data_size)
    }

    pub fn sra(dtype: ir::Dtype) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(matches!(data_size, DataSize::Word | DataSize::Double));

        Self::Sra(data_size)
    }

    pub fn mul(dtype: ir::Dtype) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(matches!(data_size, DataSize::Word | DataSize::Double));

        Self::Mul(data_size)
    }

    pub fn div(dtype: ir::Dtype, is_signed: bool) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(matches!(data_size, DataSize::Word | DataSize::Double));

        Self::Div {
            data_size,
            is_signed,
        }
    }

    pub fn rem(dtype: ir::Dtype, is_signed: bool) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(matches!(data_size, DataSize::Word | DataSize::Double));

        Self::Rem {
            data_size,
            is_signed,
        }
    }

    pub fn fadd(dtype: ir::Dtype) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(data_size.is_floating_point());

        Self::Fadd(data_size)
    }

    pub fn fsub(dtype: ir::Dtype) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(data_size.is_floating_point());

        Self::Fsub(data_size)
    }

    pub fn fmul(dtype: ir::Dtype) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(data_size.is_floating_point());

        Self::Fmul(data_size)
    }

    pub fn fdiv(dtype: ir::Dtype) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(data_size.is_floating_point());

        Self::Fdiv(data_size)
    }

    pub fn feq(dtype: ir::Dtype) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(data_size.is_floating_point());

        Self::Feq(data_size)
    }

    pub fn flt(dtype: ir::Dtype) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(data_size.is_floating_point());

        Self::Flt(data_size)
    }

    pub fn fmv_int_to_float(dtype: ir::Dtype) -> Self {
        let float_data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(float_data_size.is_floating_point());

        Self::FmvIntToFloat { float_data_size }
    }

    pub fn fmv_float_to_int(dtype: ir::Dtype) -> Self {
        let float_data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(float_data_size.is_floating_point());

        Self::FmvFloatToInt { float_data_size }
    }

    pub fn fcvt_int_to_float(from: ir::Dtype, to: ir::Dtype) -> Self {
        let is_signed = from.is_int_signed();
        let int_data_size =
            DataSize::try_from(from).expect("`data_size` must be derived from `dtype`");
        assert!(matches!(int_data_size, DataSize::Word | DataSize::Double));

        let float_data_size =
            DataSize::try_from(to).expect("`data_size` must be derived from `dtype`");
        assert!(float_data_size.is_floating_point());

        Self::FcvtIntToFloat {
            int_data_size,
            float_data_size,
            is_signed,
        }
    }

    pub fn fcvt_float_to_int(from: ir::Dtype, to: ir::Dtype) -> Self {
        let float_data_size =
            DataSize::try_from(from).expect("`data_size` must be derived from `dtype`");
        assert!(float_data_size.is_floating_point());

        let is_signed = to.is_int_signed();
        let int_data_size =
            DataSize::try_from(to).expect("`data_size` must be derived from `dtype`");
        assert!(matches!(int_data_size, DataSize::Word | DataSize::Double));

        Self::FcvtFloatToInt {
            float_data_size,
            int_data_size,
            is_signed,
        }
    }
}

impl fmt::Display for RType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Add(data_size) => write!(f, "add{}", data_size.word().write_string()),
            Self::Sub(data_size) => write!(f, "sub{}", data_size.word().write_string()),
            Self::Sll(data_size) => write!(f, "sll{}", data_size.word().write_string()),
            Self::Srl(data_size) => write!(f, "srl{}", data_size.word().write_string()),
            Self::Sra(data_size) => write!(f, "sra{}", data_size.word().write_string()),
            Self::Mul(data_size) => write!(f, "mul{}", data_size.word().write_string()),
            Self::Div {
                data_size,
                is_signed,
            } => write!(
                f,
                "div{}{}",
                if *is_signed { "" } else { "u" },
                data_size.word().write_string()
            ),
            Self::Rem {
                data_size,
                is_signed,
            } => write!(
                f,
                "rem{}{}",
                if *is_signed { "" } else { "u" },
                data_size.word().write_string()
            ),
            Self::Slt { is_signed } => write!(f, "slt{}", if *is_signed { "" } else { "u" }),
            Self::Xor => write!(f, "xor"),
            Self::Or => write!(f, "or"),
            Self::And => write!(f, "and"),
            Self::Fadd(data_size) => write!(f, "fadd.{}", data_size),
            Self::Fsub(data_size) => write!(f, "fsub.{}", data_size),
            Self::Fmul(data_size) => write!(f, "fmul.{}", data_size),
            Self::Fdiv(data_size) => write!(f, "fdiv.{}", data_size),
            Self::Feq(data_size) => write!(f, "feq.{}", data_size),
            Self::Flt(data_size) => write!(f, "flt.{}", data_size),
            Self::FmvFloatToInt { float_data_size } => {
                assert!(float_data_size.is_floating_point());
                write!(
                    f,
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
                write!(
                    f,
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
                assert!(matches!(int_data_size, DataSize::Word | DataSize::Double));
                write!(
                    f,
                    "fcvt.{}{}.{}",
                    if matches!(int_data_size, DataSize::Word) {
                        "w"
                    } else {
                        "l"
                    },
                    if *is_signed { "" } else { "u" },
                    float_data_size
                )
            }
            Self::FcvtIntToFloat {
                int_data_size,
                float_data_size,
                is_signed,
            } => {
                assert!(float_data_size.is_floating_point());
                assert!(matches!(int_data_size, DataSize::Word | DataSize::Double));
                write!(
                    f,
                    "fcvt.{}.{}{}",
                    float_data_size,
                    if matches!(int_data_size, DataSize::Word) {
                        "w"
                    } else {
                        "l"
                    },
                    if *is_signed { "" } else { "u" }
                )
            }
            Self::FcvtFloatToFloat { from, to } => {
                assert!(from.is_floating_point());
                assert!(to.is_floating_point());
                write!(f, "fcvt.{}.{}", to, from)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IType {
    Load {
        data_size: DataSize,
        is_signed: bool,
    },
    Addi(DataSize),
    Xori,
    Ori,
    Andi,
    Slli(DataSize),
    Srli(DataSize),
    Srai(DataSize),
    Slti {
        is_signed: bool,
    },
}

impl IType {
    pub const LW: Self = Self::Load {
        data_size: DataSize::Word,
        is_signed: true,
    };
    pub const LD: Self = Self::Load {
        data_size: DataSize::Double,
        is_signed: true,
    };
    pub const ADDI: Self = Self::Addi(DataSize::Double);

    pub fn load(dtype: ir::Dtype) -> Self {
        let data_size =
            DataSize::try_from(dtype.clone()).expect("`data_size` must be derived from `dtype`");

        let is_signed = if dtype.get_int_width().is_some() {
            dtype.is_int_signed()
        } else {
            false
        };

        let is_signed = if data_size == DataSize::Double {
            true
        } else {
            is_signed
        };

        Self::Load {
            data_size,
            is_signed,
        }
    }

    pub fn slli(dtype: ir::Dtype) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(matches!(data_size, DataSize::Word | DataSize::Double));

        Self::Slli(data_size)
    }

    pub fn srli(dtype: ir::Dtype) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(matches!(data_size, DataSize::Word | DataSize::Double));

        Self::Srli(data_size)
    }

    pub fn srai(dtype: ir::Dtype) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(matches!(data_size, DataSize::Word | DataSize::Double));

        Self::Srai(data_size)
    }
}

impl fmt::Display for IType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Load {
                data_size,
                is_signed,
            } => {
                if data_size.is_integer() {
                    write!(f, "l{}{}", data_size, if *is_signed { "" } else { "u" })
                } else {
                    write!(
                        f,
                        "fl{}",
                        if *data_size == DataSize::SinglePrecision {
                            "w"
                        } else {
                            "d"
                        }
                    )
                }
            }
            Self::Addi(data_size) => write!(f, "addi{}", data_size.word().write_string()),
            Self::Xori => write!(f, "xori"),
            Self::Ori => write!(f, "ori"),
            Self::Andi => write!(f, "andi"),
            Self::Slli(data_size) => write!(f, "slli{}", data_size.word().write_string()),
            Self::Srli(data_size) => write!(f, "srli{}", data_size.word().write_string()),
            Self::Srai(data_size) => write!(f, "srai{}", data_size.word().write_string()),
            Self::Slti { is_signed } => write!(f, "slti{}", if *is_signed { "" } else { "u" }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SType {
    Store(DataSize),
}

impl SType {
    pub const SW: Self = Self::Store(DataSize::Word);
    pub const SD: Self = Self::Store(DataSize::Double);

    pub fn store(dtype: ir::Dtype) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        Self::Store(data_size)
    }
}

impl fmt::Display for SType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Store(data_size) => {
                if data_size.is_integer() {
                    write!(f, "s{}", data_size)
                } else {
                    write!(
                        f,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BType {
    Beq,
    Bne,
    Blt { is_signed: bool },
    Bge { is_signed: bool },
}

impl fmt::Display for BType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Beq => "beq".to_string(),
                Self::Bne => "bne".to_string(),
                Self::Blt { is_signed } => format!("blt{}", if *is_signed { "" } else { "u" }),
                Self::Bge { is_signed } => format!("bge{}", if *is_signed { "" } else { "u" }),
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UType {
    Lui,
}

impl fmt::Display for UType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lui => write!(f, "lui"),
        }
    }
}

/// The assembler implements several convenience psuedo-instructions that are formed from multiple
/// instructions in the base ISA, but have implicit arguments or reversed arguments that result in
/// distinct semantics.
///
/// For more information:
/// - <https://github.com/michaeljclark/michaeljclark.github.io/blob/master/asm.md#assembler-pseudo-instructions>
/// - <https://riscv.org/wp-content/uploads/2017/05/riscv-spec-v2.2.pdf> (110p)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pseudo {
    /// la rd, symbol
    La { rd: Register, symbol: Label },
    /// li rd, immediate
    Li {
        rd: Register,
        // TODO: consider architecture dependency (current: 64-bit architecture)
        imm: u64,
    },
    /// mv rd, rs
    Mv { rd: Register, rs: Register },
    /// fmv.s rd, rs or fmv.d rd, rs
    Fmv {
        data_size: DataSize,
        rd: Register,
        rs: Register,
    },
    /// neg(w) rd, rs
    Neg {
        data_size: DataSize,
        rd: Register,
        rs: Register,
    },
    /// sext.w rd, rs
    SextW { rd: Register, rs: Register },
    /// seqz rd, rs
    Seqz { rd: Register, rs: Register },
    /// snez rd, rs
    Snez { rd: Register, rs: Register },
    /// fneg.s rd, rs or fneg.d rd, rs
    Fneg {
        data_size: DataSize,
        rd: Register,
        rs: Register,
    },
    /// j offset
    J { offset: Label },
    /// jr rs
    Jr { rs: Register },
    /// jalr rs
    Jalr { rs: Register },
    /// ret
    Ret,
    /// call offset
    Call { offset: Label },
}

impl Pseudo {
    pub fn neg(dtype: ir::Dtype, rd: Register, rs: Register) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(matches!(data_size, DataSize::Word | DataSize::Double));

        Self::Neg { data_size, rd, rs }
    }

    pub fn fneg(dtype: ir::Dtype, rd: Register, rs: Register) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(data_size.is_floating_point());

        Self::Fneg { data_size, rd, rs }
    }
}

impl fmt::Display for Pseudo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::La { rd, symbol } => write!(f, "la\t{},{}", rd, symbol),
            Self::Li { rd, imm } => write!(f, "li\t{},{}", rd, *imm as i64),
            Self::Mv { rd, rs } => write!(f, "mv\t{},{}", rd, rs),
            Self::Fmv { data_size, rd, rs } => write!(f, "fmv.{}\t{},{}", data_size, rd, rs),
            Self::Neg { data_size, rd, rs } => {
                write!(f, "neg{}\t{},{}", data_size.word().write_string(), rd, rs)
            }
            Self::SextW { rs, rd } => {
                write!(f, "sext.w\t{},{}", rd, rs)
            }
            Self::Seqz { rd, rs } => write!(f, "seqz\t{},{}", rd, rs),
            Self::Snez { rd, rs } => write!(f, "snez\t{},{}", rd, rs),
            Self::Fneg { data_size, rd, rs } => write!(f, "fneg.{}\t{},{}", data_size, rd, rs),
            Self::J { offset } => write!(f, "j\t{}", offset),
            Self::Jr { rs } => write!(f, "jr\t{}", rs),
            Self::Jalr { rs } => write!(f, "jalr\t{}", rs),
            Self::Ret => write!(f, "ret"),
            Self::Call { offset } => write!(f, "call\t{}", offset),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Immediate {
    // TODO: consider architecture dependency (current: 64-bit architecture)
    Value(u64),
    /// %hi(symbol) or %lo(symbol)
    Relocation {
        relocation: RelocationFunction,
        symbol: Label,
    },
}

impl Immediate {
    pub fn relocation(relocation: RelocationFunction, symbol: Label) -> Self {
        Self::Relocation { relocation, symbol }
    }
}

impl fmt::Display for Immediate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Value(value) => format!("{}", *value as i64),
                Self::Relocation { relocation, symbol } => {
                    format!("{}({})", relocation, symbol)
                }
            }
        )
    }
}

/// The relocation function creates synthesize operand values that are resolved
/// at program link time and are used as immediate parameters for specific instructions.
///
/// For more details: <https://github.com/riscv-non-isa/riscv-asm-manual/blob/master/riscv-asm.md>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelocationFunction {
    /// %hi
    Hi20,
    /// %lo
    Lo12,
}

impl fmt::Display for RelocationFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Hi20 => "%hi",
                Self::Lo12 => "%lo",
            }
        )
    }
}

/// `Label` is used as branch, unconditional jump targets and symbol offsets.
///
/// For more details: <https://github.com/michaeljclark/michaeljclark.github.io/blob/master/asm.md#labels>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Label(pub String);

impl Label {
    pub fn new(name: &str, block_id: ir::BlockId) -> Self {
        let id = block_id.0;
        Self(format!(".{}_L{}", name, id))
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataSize {
    Byte,
    Half,
    Word,
    Double,
    SinglePrecision,
    DoublePrecision,
}

impl TryFrom<ir::Dtype> for DataSize {
    type Error = ();

    fn try_from(dtype: ir::Dtype) -> Result<Self, Self::Error> {
        let (size, is_float) = match dtype {
            ir::Dtype::Int { width, .. } => {
                let size = (width - 1) / ir::Dtype::BITS_OF_BYTE + 1;
                (size, false)
            }
            ir::Dtype::Float { width, .. } => {
                let size = (width - 1) / ir::Dtype::BITS_OF_BYTE + 1;
                (size, true)
            }
            ir::Dtype::Pointer { .. } => (ir::Dtype::SIZE_OF_POINTER, false),
            _ => todo!("DataSize::try_from: support dtype: {:?}", dtype),
        };

        let align = match (size, is_float) {
            (ir::Dtype::SIZE_OF_CHAR, false) => Self::Byte,
            (ir::Dtype::SIZE_OF_SHORT, false) => Self::Half,
            (ir::Dtype::SIZE_OF_INT, false) => Self::Word,
            (ir::Dtype::SIZE_OF_LONG, false) => Self::Double,
            (ir::Dtype::SIZE_OF_FLOAT, true) => Self::SinglePrecision,
            (ir::Dtype::SIZE_OF_DOUBLE, true) => Self::DoublePrecision,
            _ => panic!("there is no other possible case"),
        };

        Ok(align)
    }
}

impl DataSize {
    pub fn is_integer(&self) -> bool {
        matches!(self, Self::Byte | Self::Half | Self::Word | Self::Double)
    }

    pub fn is_floating_point(&self) -> bool {
        matches!(self, Self::SinglePrecision | Self::DoublePrecision)
    }

    fn word(self) -> Option<Self> {
        if self == DataSize::Word {
            Some(self)
        } else {
            None
        }
    }
}

impl fmt::Display for DataSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Byte => "b",
                Self::Half => "h",
                Self::Word => "w",
                Self::Double => "d",
                Self::SinglePrecision => "s",
                Self::DoublePrecision => "d",
            }
        )
    }
}

/// ABI name for RISC-V integer and floating-point register
///
/// For more details: <https://content.riscv.org/wp-content/uploads/2017/05/riscv-spec-v2.2.pdf> (109p)
// TODO: Add calling convention information (caller/callee-save registers)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum Register {
    Zero,
    Ra,
    Sp,
    Gp,
    Tp,
    /// E.g., t0
    Temp(RegisterType, usize),
    /// E.g., s0
    Saved(RegisterType, usize),
    /// E.g., a0
    Arg(RegisterType, usize),
}

impl Register {
    pub const T0: Self = Self::Temp(RegisterType::Integer, 0);
    pub const T1: Self = Self::Temp(RegisterType::Integer, 1);
    pub const T2: Self = Self::Temp(RegisterType::Integer, 2);
    pub const T3: Self = Self::Temp(RegisterType::Integer, 3);
    pub const T4: Self = Self::Temp(RegisterType::Integer, 4);
    pub const T5: Self = Self::Temp(RegisterType::Integer, 5);
    pub const T6: Self = Self::Temp(RegisterType::Integer, 6);

    pub const S0: Self = Self::Saved(RegisterType::Integer, 0);
    pub const S1: Self = Self::Saved(RegisterType::Integer, 1);
    pub const S2: Self = Self::Saved(RegisterType::Integer, 2);
    pub const S3: Self = Self::Saved(RegisterType::Integer, 3);
    pub const S4: Self = Self::Saved(RegisterType::Integer, 4);
    pub const S5: Self = Self::Saved(RegisterType::Integer, 5);
    pub const S6: Self = Self::Saved(RegisterType::Integer, 6);
    pub const S7: Self = Self::Saved(RegisterType::Integer, 7);
    pub const S8: Self = Self::Saved(RegisterType::Integer, 8);
    pub const S9: Self = Self::Saved(RegisterType::Integer, 9);
    pub const S10: Self = Self::Saved(RegisterType::Integer, 10);
    pub const S11: Self = Self::Saved(RegisterType::Integer, 11);

    pub const A0: Self = Self::Arg(RegisterType::Integer, 0);
    pub const A1: Self = Self::Arg(RegisterType::Integer, 1);
    pub const A2: Self = Self::Arg(RegisterType::Integer, 2);
    pub const A3: Self = Self::Arg(RegisterType::Integer, 3);
    pub const A4: Self = Self::Arg(RegisterType::Integer, 4);
    pub const A5: Self = Self::Arg(RegisterType::Integer, 5);
    pub const A6: Self = Self::Arg(RegisterType::Integer, 6);
    pub const A7: Self = Self::Arg(RegisterType::Integer, 7);

    pub const FT0: Self = Self::Temp(RegisterType::FloatingPoint, 0);
    pub const FT1: Self = Self::Temp(RegisterType::FloatingPoint, 1);
    pub const FT2: Self = Self::Temp(RegisterType::FloatingPoint, 2);
    pub const FT3: Self = Self::Temp(RegisterType::FloatingPoint, 3);
    pub const FT4: Self = Self::Temp(RegisterType::FloatingPoint, 4);
    pub const FT5: Self = Self::Temp(RegisterType::FloatingPoint, 5);
    pub const FT6: Self = Self::Temp(RegisterType::FloatingPoint, 6);
    pub const FT7: Self = Self::Temp(RegisterType::FloatingPoint, 7);
    pub const FT8: Self = Self::Temp(RegisterType::FloatingPoint, 8);
    pub const FT9: Self = Self::Temp(RegisterType::FloatingPoint, 9);
    pub const FT10: Self = Self::Temp(RegisterType::FloatingPoint, 10);
    pub const FT11: Self = Self::Temp(RegisterType::FloatingPoint, 11);

    pub const FS0: Self = Self::Saved(RegisterType::FloatingPoint, 0);
    pub const FS1: Self = Self::Saved(RegisterType::FloatingPoint, 1);
    pub const FS2: Self = Self::Saved(RegisterType::FloatingPoint, 2);
    pub const FS3: Self = Self::Saved(RegisterType::FloatingPoint, 3);
    pub const FS4: Self = Self::Saved(RegisterType::FloatingPoint, 4);
    pub const FS5: Self = Self::Saved(RegisterType::FloatingPoint, 5);
    pub const FS6: Self = Self::Saved(RegisterType::FloatingPoint, 6);
    pub const FS7: Self = Self::Saved(RegisterType::FloatingPoint, 7);
    pub const FS8: Self = Self::Saved(RegisterType::FloatingPoint, 8);
    pub const FS9: Self = Self::Saved(RegisterType::FloatingPoint, 9);
    pub const FS10: Self = Self::Saved(RegisterType::FloatingPoint, 10);
    pub const FS11: Self = Self::Saved(RegisterType::FloatingPoint, 11);

    pub const FA0: Self = Self::Arg(RegisterType::FloatingPoint, 0);
    pub const FA1: Self = Self::Arg(RegisterType::FloatingPoint, 1);
    pub const FA2: Self = Self::Arg(RegisterType::FloatingPoint, 2);
    pub const FA3: Self = Self::Arg(RegisterType::FloatingPoint, 3);
    pub const FA4: Self = Self::Arg(RegisterType::FloatingPoint, 4);
    pub const FA5: Self = Self::Arg(RegisterType::FloatingPoint, 5);
    pub const FA6: Self = Self::Arg(RegisterType::FloatingPoint, 6);
    pub const FA7: Self = Self::Arg(RegisterType::FloatingPoint, 7);

    pub fn temp(register_type: RegisterType, id: usize) -> Self {
        match register_type {
            RegisterType::Integer => assert!(id <= 6),
            RegisterType::FloatingPoint => assert!(id <= 11),
        }

        Self::Temp(register_type, id)
    }

    pub fn saved(register_type: RegisterType, id: usize) -> Self {
        assert!(id <= 11);
        Self::Saved(register_type, id)
    }

    pub fn arg(register_type: RegisterType, id: usize) -> Self {
        assert!(id <= 7);
        Self::Arg(register_type, id)
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let register = match self {
            Self::Zero => "zero".to_string(),
            Self::Ra => "ra".to_string(),
            Self::Sp => "sp".to_string(),
            Self::Gp => "gp".to_string(),
            Self::Tp => "tp".to_string(),
            Self::Temp(register_type, id) => format!("{}t{}", register_type, id),
            Self::Saved(register_type, id) => format!("{}s{}", register_type, id),
            Self::Arg(register_type, id) => format!("{}a{}", register_type, id),
        };
        write!(f, "{}", register)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum RegisterType {
    Integer,
    FloatingPoint,
}

impl fmt::Display for RegisterType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::FloatingPoint => "f",
                Self::Integer => "",
            },
        )
    }
}
