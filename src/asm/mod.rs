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
    pub label: Label,
    pub directives: Vec<Directive>,
}

impl Variable {
    pub fn new(label: Label, directives: Vec<Directive>) -> Self {
        Self { label, directives }
    }
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

#[derive(Debug, Clone, PartialEq)]
pub enum SectionType {
    Text,
    Data,
    Rodata,
    Bss,
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
        rs2: Option<Register>,
    },
    /// I-type instruction format
    /// https://riscv.org/specifications/isa-spec-pdf/ (16p, 129p)
    IType {
        instr: IType,
        rd: Register,
        rs1: Register,
        imm: Immediate,
    },
    /// S-type instruction format
    /// https://riscv.org/specifications/isa-spec-pdf/ (16p, 129p)
    SType {
        instr: SType,
        rs1: Register,
        rs2: Register,
        imm: Immediate,
    },
    /// B-type instruction format
    /// https://riscv.org/specifications/isa-spec-pdf/ (16p, 129p)
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
    Sll(Option<DataSize>),
    Srl(Option<DataSize>),
    Sra(Option<DataSize>),
    Mul(Option<DataSize>),
    Div {
        data_size: Option<DataSize>,
        is_signed: bool,
    },
    Rem {
        data_size: Option<DataSize>,
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
        int_data_size: Option<DataSize>,
        float_data_size: DataSize,
        is_signed: bool,
    },
    /// fcvt.l(u).s or fcvt.l(u).d
    /// fcvt.w(u).s or fcvt.w(u).d
    FcvtFloatToInt {
        float_data_size: DataSize,
        int_data_size: Option<DataSize>,
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
        assert!(data_size.is_integer());

        Self::Add(data_size.word())
    }

    pub fn sub(dtype: ir::Dtype) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(data_size.is_integer());

        Self::Sub(data_size.word())
    }

    pub fn sll(dtype: ir::Dtype) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(data_size.is_integer());

        Self::Sll(data_size.word())
    }

    pub fn srl(dtype: ir::Dtype) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(data_size.is_integer());

        Self::Srl(data_size.word())
    }

    pub fn sra(dtype: ir::Dtype) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(data_size.is_integer());

        Self::Sra(data_size.word())
    }

    pub fn mul(dtype: ir::Dtype) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(data_size.is_integer());

        Self::Mul(data_size.word())
    }

    pub fn div(dtype: ir::Dtype, is_signed: bool) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(data_size.is_integer());

        Self::Div {
            data_size: data_size.word(),
            is_signed,
        }
    }

    pub fn rem(dtype: ir::Dtype, is_signed: bool) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(data_size.is_integer());

        Self::Rem {
            data_size: data_size.word(),
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
        assert!(int_data_size.is_integer());

        let float_data_size =
            DataSize::try_from(to).expect("`data_size` must be derived from `dtype`");
        assert!(float_data_size.is_floating_point());

        Self::FcvtIntToFloat {
            int_data_size: int_data_size.word(),
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
        assert!(int_data_size.is_integer());

        Self::FcvtFloatToInt {
            float_data_size,
            int_data_size: int_data_size.word(),
            is_signed,
        }
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
    Load {
        data_size: DataSize,
        is_signed: bool,
    },
    Addi(Option<DataSize>),
    Xori,
    Ori,
    Andi,
    Slli(Option<DataSize>),
    Srli(Option<DataSize>),
    Srai(Option<DataSize>),
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
    pub const ADDI: Self = Self::Addi(None);

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
        assert!(data_size.is_integer());

        Self::Slli(data_size.word())
    }

    pub fn srli(dtype: ir::Dtype) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(data_size.is_integer());

        Self::Srli(data_size.word())
    }

    pub fn srai(dtype: ir::Dtype) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(data_size.is_integer());

        Self::Srai(data_size.word())
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
    Blt { is_signed: bool },
    Bge { is_signed: bool },
}

#[derive(Debug, Clone, PartialEq)]
pub enum UType {
    Lui,
}

/// The assembler implements a number of convenience psuedo-instructions that are formed from
/// instructions in the base ISA, but have implicit arguments or in some case reversed arguments,
/// that result in distinct semantics.
/// https://github.com/rv8-io/rv8-io.github.io/blob/master/asm.md#assembler-pseudo-instructions
/// https://riscv.org/specifications/isa-spec-pdf/ (139p)
#[derive(Debug, Clone, PartialEq)]
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
        data_size: Option<DataSize>,
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
        let data_size = if data_size == DataSize::Word {
            Some(data_size)
        } else {
            None
        };

        Self::Neg { data_size, rd, rs }
    }

    pub fn fneg(dtype: ir::Dtype, rd: Register, rs: Register) -> Self {
        let data_size =
            DataSize::try_from(dtype).expect("`data_size` must be derived from `dtype`");
        assert!(data_size.is_floating_point());

        Self::Fneg { data_size, rd, rs }
    }
}

#[derive(Debug, Clone, PartialEq)]
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

/// The relocation function creates synthesize operand values that are resolved
/// at program link time and are used as immediate parameters to specific instructions.
/// https://github.com/riscv/riscv-asm-manual/blob/master/riscv-asm.md
#[derive(Debug, Clone, PartialEq)]
pub enum RelocationFunction {
    /// %hi
    HI20,
    /// %lo
    LO12,
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
        match self {
            Self::Byte | Self::Half | Self::Word | Self::Double => true,
            _ => false,
        }
    }

    pub fn is_floating_point(&self) -> bool {
        match self {
            Self::SinglePrecision | Self::DoublePrecision => true,
            _ => false,
        }
    }

    fn word(self) -> Option<Self> {
        if self == DataSize::Word {
            Some(self)
        } else {
            None
        }
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum RegisterType {
    Integer,
    FloatingPoint,
}
