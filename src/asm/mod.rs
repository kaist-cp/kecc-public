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
    /// https://riscv.org/specifications/isa-spec-pdf/ (16p, 129p)
    RType {
        instr: RType,
        rd: Register,
        rs1: Register,
        rs2: Register,
    },
    /// I-type instruction format
    /// https://riscv.org/specifications/isa-spec-pdf/ (16p, 129p)
    IType {
        instr: IType,
        rd: Register,
        rs1: Register,
        imm: isize,
    },
    /// S-type instruction format
    /// https://riscv.org/specifications/isa-spec-pdf/ (16p, 129p)
    SType {
        instr: SType,
        rs1: Register,
        rs2: Register,
        imm: isize,
    },
    /// B-type instruction format
    /// https://riscv.org/specifications/isa-spec-pdf/ (16p, 129p)
    BType {
        instr: BType,
        rs1: Register,
        rs2: Register,
        imm: Label,
    },
    Pseudo(Pseudo),
}

/// If the enum variant contains `Option<DataSize>`,
/// it means that the instructions used may vary according to `DataSize`.
/// Use 'Some' if RISC-V ISA provides instruction to support a specific 'DataSize',
/// if not, 'None' which means to use default instruction.
/// Because KECC uses RV64 (RISC-V ISA for 64-bit architecture),
/// KECC uses `Some` if `DataSize` is `Word`, if not, use `None`.
/// https://riscv.org/specifications/isa-spec-pdf/ (35p)
///
/// If the enum variant contains `bool`,
/// It means that different instructions exist
/// depending on whether the operand is signed or not.
#[derive(Debug, Clone, PartialEq)]
pub enum RType {
    Add(Option<DataSize>),
    Sub(Option<DataSize>),
    Mul(Option<DataSize>),
    Div(Option<DataSize>, bool),
    Slt(bool),
    Xor,
}

impl RType {
    pub fn add(dtype: ir::Dtype) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        let data_size = if data_size == DataSize::Word {
            Some(data_size)
        } else {
            None
        };

        Self::Add(data_size)
    }

    pub fn sub(dtype: ir::Dtype) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        let data_size = if data_size == DataSize::Word {
            Some(data_size)
        } else {
            None
        };

        Self::Sub(data_size)
    }

    pub fn mul(dtype: ir::Dtype) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        let data_size = if data_size == DataSize::Word {
            Some(data_size)
        } else {
            None
        };

        Self::Mul(data_size)
    }

    pub fn div(dtype: ir::Dtype, is_signed: bool) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        let data_size = if data_size == DataSize::Word {
            Some(data_size)
        } else {
            None
        };

        Self::Div(data_size, is_signed)
    }
}

/// If the enum variant contains `Option<DataSize>`,
/// it means that the instructions used may vary according to `DataSize`.
/// Use 'Some' if RISC-V ISA provides instruction to support a specific 'DataSize',
/// if not, 'None' which means to use default instruction.
/// Because KECC uses RV64 (RISC-V ISA for 64-bit architecture),
/// KECC uses `Some` if `DataSize` is `Word`, if not, use `None`.
/// https://riscv.org/specifications/isa-spec-pdf/ (35p)
#[derive(Debug, Clone, PartialEq)]
pub enum IType {
    Load(DataSize),
    Addi(Option<DataSize>),
    Andi,
    Slli,
    Srli,
}

impl IType {
    pub const LW: Self = Self::Load(DataSize::Word);
    pub const LD: Self = Self::Load(DataSize::Double);
    pub const ADDI: Self = Self::Addi(None);

    pub fn load(dtype: ir::Dtype) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        Self::Load(data_size)
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
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        Self::Store(data_size)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BType {
    Beq,
    Bne,
    Blt(bool),
    Bge(bool),
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
    Mv { rd: Register, rs: Register },
    /// neg(w) rd, rs
    Neg {
        data_size: Option<DataSize>,
        rs: Register,
        rd: Register,
    },
    /// sext.w rd, rs
    SextW { rd: Register, rs: Register },
    /// seqz rd, rs
    Seqz { rd: Register, rs: Register },
    /// j offset
    J { offset: Label },
    /// jr rs
    Jr { rs: Register },
    /// ret
    Ret,
    /// call offset
    Call { offset: Label },
}

impl Pseudo {
    pub fn neg(dtype: ir::Dtype, rs: Register, rd: Register) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        let data_size = if data_size == DataSize::Word {
            Some(data_size)
        } else {
            None
        };

        Self::Neg { data_size, rs, rd }
    }
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
/// https://content.riscv.org/wp-content/uploads/2017/05/riscv-spec-v2.2.pdf (155p)
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
    pub const T0: Self = Self::Temp(0);
    pub const T1: Self = Self::Temp(1);
    pub const T2: Self = Self::Temp(2);
    pub const T3: Self = Self::Temp(3);
    pub const T4: Self = Self::Temp(4);
    pub const T5: Self = Self::Temp(5);
    pub const T6: Self = Self::Temp(6);

    pub const S0: Self = Self::Saved(0);
    pub const S1: Self = Self::Saved(1);
    pub const S2: Self = Self::Saved(2);
    pub const S3: Self = Self::Saved(3);
    pub const S4: Self = Self::Saved(4);
    pub const S5: Self = Self::Saved(5);
    pub const S6: Self = Self::Saved(6);
    pub const S7: Self = Self::Saved(7);
    pub const S8: Self = Self::Saved(8);
    pub const S9: Self = Self::Saved(9);
    pub const S10: Self = Self::Saved(10);
    pub const S11: Self = Self::Saved(11);

    pub const A0: Self = Self::Arg(0);
    pub const A1: Self = Self::Arg(1);
    pub const A2: Self = Self::Arg(2);
    pub const A3: Self = Self::Arg(3);
    pub const A4: Self = Self::Arg(4);
    pub const A5: Self = Self::Arg(5);
    pub const A6: Self = Self::Arg(6);
    pub const A7: Self = Self::Arg(7);

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
