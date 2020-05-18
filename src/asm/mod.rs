mod write_asm;

use crate::ir;

use core::convert::TryFrom;

#[derive(Debug, Clone, PartialEq)]
pub struct TODO {}

/// TODO
#[derive(Debug, Clone, PartialEq)]
pub struct Asm {
    pub unit: TranslationUnit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TranslationUnit {
    pub functions: Vec<Section<Function>>,
    pub variables: Vec<Section<Variable>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Section<T> {
    /// Section Headers provice size, offset, type, alignment and flags of the sections
    /// https://github.com/rv8-io/rv8-io.github.io/blob/master/asm.md#section-header
    pub header: Vec<Directive>,
    pub body: T,
}

/// An object file is made up of multiple sections, with each section corresponding to
/// distinct types of executable code or data.
/// https://github.com/rv8-io/rv8-io.github.io/blob/master/asm.md#sections
impl<T> Section<T> {
    pub fn new(header: Vec<Directive>, body: T) -> Self {
        Self { header, body }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub blocks: Vec<Block>,
}

impl Function {
    pub fn new(blocks: Vec<Block>) -> Self {
        Self { blocks }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    todo: TODO,
}

#[derive(Debug, Clone, PartialEq)]
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

/// The assembler implements a number of directives that control the assembly of instructions
/// into an object file.
/// https://github.com/rv8-io/rv8-io.github.io/blob/master/asm.md#assembler-directives
#[derive(Debug, Clone, PartialEq)]
pub enum Directive {
    /// .globl symbol
    Globl(Label),
    /// .type symbol, symbol_type
    Type(Label, SymbolType),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolType {
    Function,
    Object,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    /// R-type instruction format
    /// https://content.riscv.org/wp-content/uploads/2017/05/riscv-spec-v2.2.pdf (11p, 104p)
    RType {
        instr: RType,
        rd: Register,
        rs1: Register,
        rs2: Register,
    },
    /// I-type instruction format
    /// https://content.riscv.org/wp-content/uploads/2017/05/riscv-spec-v2.2.pdf (11p, 104p)
    IType {
        instr: IType,
        rd: Register,
        rs1: Register,
        imm: isize,
    },
    /// S-type instruction format
    /// https://content.riscv.org/wp-content/uploads/2017/05/riscv-spec-v2.2.pdf (11p, 104p)
    SType {
        instr: SType,
        rs1: Register,
        rs2: Register,
        imm: isize,
    },
    Pseudo(Pseudo),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RType {
    Add(Option<DataSize>),
    Mul(Option<DataSize>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum IType {
    Load(DataSize),
    Addi(Option<DataSize>),
}

impl IType {
    pub const LW: Self = Self::Load(DataSize::Word);
    pub const LD: Self = Self::Load(DataSize::Double);
    pub const ADDI: Self = Self::Addi(None);

    pub fn load(dtype: ir::Dtype) -> Self {
        let data_align =
            DataSize::try_from(dtype).expect("`data_align` must be derived from `dtype`");
        Self::Load(data_align)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SType {
    Store(DataSize),
}

impl SType {
    pub const SW: Self = Self::Store(DataSize::Word);
    pub const SD: Self = Self::Store(DataSize::Double);

    pub fn store(dtype: ir::Dtype) -> Self {
        let data_align =
            DataSize::try_from(dtype).expect("`data_align` must be derived from `dtype`");
        Self::Store(data_align)
    }
}

/// The assembler implements a number of convenience psuedo-instructions that are formed from
/// instructions in the base ISA, but have implicit arguments or in some case reversed arguments,
/// that result in distinct semantics.
/// https://github.com/rv8-io/rv8-io.github.io/blob/master/asm.md#assembler-pseudo-instructions
/// https://content.riscv.org/wp-content/uploads/2017/05/riscv-spec-v2.2.pdf (110p)
#[derive(Debug, Clone, PartialEq)]
pub enum Pseudo {
    /// li rd, immediate
    Li { rd: Register, imm: isize },
    /// mv rd, rs
    Mv { rs: Register, rd: Register },
    /// sext.w rd, rs
    SextW { rs: Register, rd: Register },
    /// j offset
    J { offset: Label },
    /// jr rs
    Jr { rs: Register },
    /// ret
    Ret,
    /// call offset
    Call { offset: Label },
}

/// `Label` is used as branch, unconditional jump targets and symbol offsets.
/// https://github.com/rv8-io/rv8-io.github.io/blob/master/asm.md#labels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Label(pub String);

impl Label {
    pub fn new(name: &str, block_id: ir::BlockId) -> Self {
        let id = block_id.0;
        Self(format!(".{}_L{}", name, id))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataSize {
    Byte,
    Half,
    Word,
    Double,
}

impl TryFrom<ir::Dtype> for DataSize {
    type Error = ();

    fn try_from(dtype: ir::Dtype) -> Result<Self, Self::Error> {
        let width = match dtype {
            ir::Dtype::Int { width, .. } => width,
            _ => todo!(),
        };

        let size = (width - 1) / ir::Dtype::BITS_OF_BYTE + 1;
        let align = match size {
            ir::Dtype::SIZE_OF_CHAR => Self::Byte,
            ir::Dtype::SIZE_OF_SHORT => Self::Half,
            ir::Dtype::SIZE_OF_INT => Self::Word,
            ir::Dtype::SIZE_OF_LONG => Self::Double,
            _ => panic!("there is no other possible case"),
        };

        Ok(align)
    }
}

// TODO: Add calling convention information (caller/callee-save registers)
/// ABI name for RISC-V integer and floating-point register
/// https://content.riscv.org/wp-content/uploads/2017/05/riscv-spec-v2.2.pdf (109p)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum Register {
    Zero,
    Ra,
    Sp,
    Gp,
    Tp,
    /// E.g., t0
    Temp(usize),
    /// E.g., s0
    Saved(usize),
    /// E.g., a0
    Arg(usize),
}

impl Register {
    // TODO: add all possible registers in the future
    pub const S0: Self = Self::Saved(0);
    pub const A0: Self = Self::Arg(0);
    pub const A5: Self = Self::Arg(5);

    pub fn temp(id: usize) -> Self {
        assert!(id <= 6);
        Self::Temp(id)
    }

    pub fn saved(id: usize) -> Self {
        assert!(id <= 11);
        Self::Saved(id)
    }

    pub fn arg(id: usize) -> Self {
        assert!(id <= 7);
        Self::Arg(id)
    }
}
