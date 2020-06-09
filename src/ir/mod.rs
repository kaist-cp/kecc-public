mod dtype;
mod interp;
mod parse;
mod write_ir;

use core::convert::TryFrom;
use core::fmt;
use core::ops::{Deref, DerefMut};
use hexf::{parse_hexf32, parse_hexf64};
use lang_c::ast;
use ordered_float::OrderedFloat;
use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};

pub use dtype::{Dtype, DtypeError, HasDtype};
pub use interp::{interp, Value};
pub use parse::Parse;

#[derive(Debug, Clone, PartialEq)]
pub struct TranslationUnit {
    pub decls: BTreeMap<String, Declaration>,
    pub structs: HashMap<String, Option<Dtype>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    Variable {
        dtype: Dtype,
        initializer: Option<ast::Initializer>,
    },
    Function {
        signature: FunctionSignature,
        definition: Option<FunctionDefinition>,
    },
}

impl TryFrom<Dtype> for Declaration {
    type Error = DtypeError;

    /// Create an appropriate declaration according to `dtype`.
    ///
    /// # Example
    ///
    /// If `int g = 0;` is declared, `dtype` is
    /// `ir::Dtype::Int{ width:32, is_signed:true, is_const:false }`.
    /// In this case, `ir::Declaration::Variable{ dtype, initializer: Some(Constant::I32(1)) }`
    /// is generated.
    ///
    /// Conversely, if `int foo();` is declared, `dtype` is
    /// `ir::Dtype::Function{ret: Scalar(Int), params: []}`.
    /// Thus, in this case, `ir::Declaration::Function` is generated.
    fn try_from(dtype: Dtype) -> Result<Self, Self::Error> {
        match &dtype {
            Dtype::Unit { .. } => Err(DtypeError::Misc {
                message: "A variable of type `void` cannot be declared".to_string(),
            }),
            Dtype::Int { .. }
            | Dtype::Float { .. }
            | Dtype::Pointer { .. }
            | Dtype::Array { .. }
            | Dtype::Struct { .. } => Ok(Declaration::Variable {
                dtype,
                initializer: None,
            }),
            Dtype::Function { .. } => Ok(Declaration::Function {
                signature: FunctionSignature::new(dtype),
                definition: None,
            }),
            Dtype::Typedef { .. } => panic!("typedef should be replaced by real dtype"),
        }
    }
}

impl Declaration {
    pub fn get_variable(&self) -> Option<(&Dtype, &Option<ast::Initializer>)> {
        if let Self::Variable { dtype, initializer } = self {
            Some((dtype, initializer))
        } else {
            None
        }
    }

    pub fn get_function(&self) -> Option<(&FunctionSignature, &Option<FunctionDefinition>)> {
        if let Self::Function {
            signature,
            definition,
        } = self
        {
            Some((signature, definition))
        } else {
            None
        }
    }

    pub fn get_function_mut(
        &mut self,
    ) -> Option<(&mut FunctionSignature, &mut Option<FunctionDefinition>)> {
        if let Self::Function {
            signature,
            definition,
        } = self
        {
            Some((signature, definition))
        } else {
            None
        }
    }

    /// Check if type is conflicting for pre-declared one
    pub fn is_compatible(&self, other: &Declaration) -> bool {
        match (self, other) {
            (Self::Variable { dtype, .. }, Self::Variable { dtype: other, .. }) => dtype == other,
            (
                Self::Function { signature, .. },
                Self::Function {
                    signature: other, ..
                },
            ) => signature.dtype() == other.dtype(),
            _ => false,
        }
    }
}

impl HasDtype for Declaration {
    fn dtype(&self) -> Dtype {
        match self {
            Self::Variable { dtype, .. } => dtype.clone(),
            Self::Function { signature, .. } => signature.dtype(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionSignature {
    pub ret: Dtype,
    pub params: Vec<Dtype>,
}

impl FunctionSignature {
    pub fn new(dtype: Dtype) -> Self {
        let (ret, params) = dtype
            .get_function_inner()
            .expect("function signature's dtype must be function type");
        Self {
            ret: ret.clone(),
            params: params.clone(),
        }
    }
}

impl HasDtype for FunctionSignature {
    fn dtype(&self) -> Dtype {
        Dtype::function(self.ret.clone(), self.params.clone())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDefinition {
    /// Memory allocations for local variables.  The allocation is performed at the beginning of a
    /// function invocation.
    pub allocations: Vec<Named<Dtype>>,

    /// Basic blocks.
    pub blocks: BTreeMap<BlockId, Block>,

    /// The initial block id.
    pub bid_init: BlockId,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct BlockId(pub usize);

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "b{}", self.0)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub phinodes: Vec<Named<Dtype>>,
    pub instructions: Vec<Named<Instruction>>,
    pub exit: BlockExit,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Instruction {
    Nop,
    BinOp {
        op: ast::BinaryOperator,
        lhs: Operand,
        rhs: Operand,
        dtype: Dtype,
    },
    UnaryOp {
        op: ast::UnaryOperator,
        operand: Operand,
        dtype: Dtype,
    },
    Store {
        ptr: Operand,
        value: Operand,
    },
    Load {
        ptr: Operand,
    },
    Call {
        callee: Operand,
        args: Vec<Operand>,
        return_type: Dtype,
    },
    TypeCast {
        value: Operand,
        target_dtype: Dtype,
    },
    /// `GetElementPtr` is inspired from `getelementptr` instruction of LLVM.
    /// https://llvm.org/docs/LangRef.html#i-getelementptr
    GetElementPtr {
        ptr: Operand,
        offset: Operand,
        dtype: Box<Dtype>,
    },
}

impl HasDtype for Instruction {
    fn dtype(&self) -> Dtype {
        match self {
            Self::Nop => Dtype::unit(),
            Self::BinOp { dtype, .. } => dtype.clone(),
            Self::UnaryOp { dtype, .. } => dtype.clone(),
            Self::Store { .. } => Dtype::unit(),
            Self::Load { ptr } => ptr
                .dtype()
                .get_pointer_inner()
                .expect("Load instruction must have pointer value as operand")
                .deref()
                .clone()
                .set_const(false),
            Self::Call { return_type, .. } => return_type.clone(),
            Self::TypeCast { target_dtype, .. } => target_dtype.clone(),
            Self::GetElementPtr { dtype, .. } => dtype.deref().clone(),
        }
    }
}

impl Instruction {
    pub fn is_pure(&self) -> bool {
        match self {
            Self::Store { .. } => false,
            Self::Call { .. } => false,
            _ => true,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum BlockExit {
    Jump {
        arg: JumpArg,
    },
    ConditionalJump {
        condition: Operand,
        arg_then: Box<JumpArg>,
        arg_else: Box<JumpArg>,
    },
    Switch {
        value: Operand,
        default: Box<JumpArg>,
        cases: Vec<(Constant, JumpArg)>,
    },
    Return {
        value: Operand,
    },
    Unreachable,
}

impl BlockExit {
    pub fn walk_jump_args<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut JumpArg),
    {
        match self {
            Self::Jump { arg } => f(arg),
            Self::ConditionalJump {
                arg_then, arg_else, ..
            } => {
                f(arg_then);
                f(arg_else);
            }
            Self::Switch { default, cases, .. } => {
                f(default);
                for (_, arg) in cases {
                    f(arg);
                }
            }
            Self::Return { .. } | Self::Unreachable => {}
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct JumpArg {
    pub bid: BlockId,
    pub args: Vec<Operand>,
}

impl JumpArg {
    pub fn new(bid: BlockId, args: Vec<Operand>) -> Self {
        Self { bid, args }
    }
}

impl fmt::Display for JumpArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}({})",
            self.bid,
            self.args
                .iter()
                .map(|a| format!("{}:{}", a, a.dtype()))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operand {
    Constant(Constant),
    Register { rid: RegisterId, dtype: Dtype },
}

impl Operand {
    pub fn constant(value: Constant) -> Self {
        Self::Constant(value)
    }

    pub fn register(rid: RegisterId, dtype: Dtype) -> Self {
        Self::Register { rid, dtype }
    }

    pub fn get_constant(&self) -> Option<&Constant> {
        if let Self::Constant(constant) = self {
            Some(constant)
        } else {
            None
        }
    }

    pub fn get_register(&self) -> Option<(&RegisterId, &Dtype)> {
        if let Self::Register { rid, dtype } = self {
            Some((rid, dtype))
        } else {
            None
        }
    }

    pub fn get_register_mut(&mut self) -> Option<(&mut RegisterId, &mut Dtype)> {
        if let Self::Register { rid, dtype } = self {
            Some((rid, dtype))
        } else {
            None
        }
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Constant(value) => write!(f, "{}", value),
            Self::Register { rid, .. } => write!(f, "{}", rid),
        }
    }
}

impl HasDtype for Operand {
    fn dtype(&self) -> Dtype {
        match self {
            Self::Constant(value) => value.dtype(),
            Self::Register { dtype, .. } => dtype.clone(),
        }
    }
}

#[derive(Debug, Eq, Clone)]
pub enum RegisterId {
    /// Registers holding pointers to local allocations.
    ///
    /// # Fields
    ///
    /// - `aid`: local allocation id.
    Local { aid: usize },

    /// Registers holding block arguments.
    ///
    /// # Fields
    ///
    /// - `bid`: When it is the initial block id, then it holds a function argument; otherwise, it
    ///   holds a phinode value.
    /// - `aid`: the argument index.
    Arg { bid: BlockId, aid: usize },

    /// Registers holding the results of instructions.
    ///
    /// # Fields
    ///
    /// - `bid`: the instruction's block id.
    /// - `iid`: the instruction's id in the block.
    Temp { bid: BlockId, iid: usize },
}

impl RegisterId {
    pub fn local(aid: usize) -> Self {
        Self::Local { aid }
    }

    pub fn arg(bid: BlockId, aid: usize) -> Self {
        Self::Arg { bid, aid }
    }

    pub fn temp(bid: BlockId, iid: usize) -> Self {
        Self::Temp { bid, iid }
    }

    pub fn is_const(&self, bid_init: BlockId) -> bool {
        match self {
            Self::Local { .. } => true,
            Self::Arg { bid, .. } => bid == &bid_init,
            _ => false,
        }
    }
}

impl fmt::Display for RegisterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Local { aid } => write!(f, "%l{}", aid),
            Self::Arg { bid, aid } => write!(f, "%{}:p{}", bid, aid),
            Self::Temp { bid, iid } => write!(f, "%{}:i{}", bid, iid),
        }
    }
}

impl PartialEq<RegisterId> for RegisterId {
    fn eq(&self, other: &RegisterId) -> bool {
        match (self, other) {
            (Self::Local { aid }, Self::Local { aid: other_aid }) => aid == other_aid,
            (
                Self::Arg { bid, aid },
                Self::Arg {
                    bid: other_bid,
                    aid: other_aid,
                },
            ) => bid == other_bid && aid == other_aid,
            (
                Self::Temp { bid, iid },
                Self::Temp {
                    bid: other_bid,
                    iid: other_iid,
                },
            ) => bid == other_bid && iid == other_iid,
            _ => false,
        }
    }
}

impl Hash for RegisterId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Local { aid } => aid.hash(state),
            Self::Arg { bid, aid } => {
                // TODO: needs to distinguish arg/temp?
                bid.hash(state);
                aid.hash(state);
            }
            Self::Temp { bid, iid } => {
                bid.hash(state);
                iid.hash(state);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Constant {
    Undef {
        dtype: Dtype,
    },
    Unit,
    Int {
        value: u128,
        width: usize,
        is_signed: bool,
    },
    Float {
        /// `value` may be `f32`, but it is possible to consider it as `f64`.
        ///
        /// * Casting from an f32 to an f64 is perfect and lossless (f32 -> f64)
        /// * Casting from an f64 to an f32 will produce the closest possible value (f64 -> f32)
        /// https://doc.rust-lang.org/stable/reference/expressions/operator-expr.html#type-cast-expressions
        value: OrderedFloat<f64>,
        width: usize,
    },
    GlobalVariable {
        name: String,
        dtype: Dtype,
    },
}

impl TryFrom<&ast::Constant> for Constant {
    type Error = ();

    fn try_from(constant: &ast::Constant) -> Result<Self, Self::Error> {
        match constant {
            ast::Constant::Integer(integer) => {
                let dtype = match integer.suffix.size {
                    ast::IntegerSize::Int => Dtype::INT,
                    ast::IntegerSize::Long => Dtype::LONG,
                    ast::IntegerSize::LongLong => Dtype::LONGLONG,
                };

                let pat = match integer.base {
                    ast::IntegerBase::Decimal => Self::DECIMAL,
                    ast::IntegerBase::Octal => Self::OCTAL,
                    ast::IntegerBase::Hexadecimal => Self::HEXADECIMAL,
                };
                let value = u128::from_str_radix(integer.number.deref(), pat).unwrap();

                let is_signed = !integer.suffix.unsigned && {
                    // Even if `suffix` represents `signed`, integer literal cannot be translated
                    // to minus value. For this reason, if the sign bit is on, dtype automatically
                    // transformed to `unsigned`. Let's say integer literal is `0xFFFFFFFF`,
                    // it translated to unsigned integer even though it has no `U` suffix.
                    let width = dtype.get_int_width().unwrap();
                    let threshold = 1u128 << (width as u128 - 1);
                    value < threshold
                };

                Ok(Self::int(value, dtype.set_signed(is_signed)))
            }
            ast::Constant::Float(float) => {
                let pat = match float.base {
                    ast::FloatBase::Decimal => Self::DECIMAL,
                    ast::FloatBase::Hexadecimal => Self::HEXADECIMAL,
                };

                let (dtype, value) = match float.suffix.format {
                    ast::FloatFormat::Float => {
                        // Casting from an f32 to an f64 is perfect and lossless (f32 -> f64)
                        // https://doc.rust-lang.org/stable/reference/expressions/operator-expr.html#type-cast-expressions
                        let value = match pat {
                            Self::DECIMAL => float.number.parse::<f32>().unwrap() as f64,
                            Self::HEXADECIMAL => {
                                let mut hex_number = "0x".to_string();
                                hex_number.push_str(float.number.deref());
                                parse_hexf32(&hex_number, true).unwrap() as f64
                            }
                            _ => panic!(
                                "Constant::try_from::<&ast::Constant>: \
                                 {:?} is not a pattern of `pat`",
                                pat
                            ),
                        };
                        (Dtype::FLOAT, value)
                    }
                    ast::FloatFormat::Double => {
                        let value = match pat {
                            Self::DECIMAL => float.number.parse::<f64>().unwrap(),
                            Self::HEXADECIMAL => {
                                let mut hex_number = "0x".to_string();
                                hex_number.push_str(float.number.deref());
                                parse_hexf64(&hex_number, true).unwrap()
                            }
                            _ => panic!(
                                "Constant::try_from::<&ast::Constant>: \
                                 {:?} is not a pattern of `pat`",
                                pat
                            ),
                        };
                        (Dtype::DOUBLE, value)
                    }
                    ast::FloatFormat::LongDouble => {
                        panic!("`FloatFormat::LongDouble` is_unsupported")
                    }
                    ast::FloatFormat::TS18661Format(_) => {
                        panic!("`FloatFormat::TS18661Format` is_unsupported")
                    }
                };

                Ok(Self::float(value, dtype))
            }
            ast::Constant::Character(character) => {
                let dtype = Dtype::CHAR;
                let value = character.parse::<char>().unwrap() as u128;

                Ok(Self::int(value, dtype))
            }
        }
    }
}

impl TryFrom<&ast::Expression> for Constant {
    type Error = ();

    fn try_from(expr: &ast::Expression) -> Result<Self, Self::Error> {
        match expr {
            ast::Expression::Constant(constant) => Self::try_from(&constant.node),
            ast::Expression::UnaryOperator(unary) => {
                let constant = Self::try_from(&unary.node.operand.node)?;
                // When an IR is generated, there are cases where some expressions must be
                // interpreted unconditionally as compile-time constant value. In this case,
                // we need to translate also the expression applied `minus` unary operator
                // to compile-time constant value directly.
                // Let's say expression is `case -1: { .. }`,
                // `-1` must be interpreted to compile-time constant value.
                match &unary.node.operator.node {
                    ast::UnaryOperator::Minus => Ok(constant.minus()),
                    ast::UnaryOperator::Plus => Ok(constant),
                    _ => Err(()),
                }
            }
            _ => Err(()),
        }
    }
}

impl Constant {
    const DECIMAL: u32 = 10;
    const OCTAL: u32 = 8;
    const HEXADECIMAL: u32 = 16;

    #[inline]
    pub fn is_integer_constant(&self) -> bool {
        if let Self::Int { .. } = self {
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn undef(dtype: Dtype) -> Self {
        Self::Undef { dtype }
    }

    #[inline]
    pub fn unit() -> Self {
        Self::Unit
    }

    #[inline]
    pub fn int(value: u128, dtype: Dtype) -> Self {
        let width = dtype.get_int_width().expect("`dtype` must be `Dtype::Int`");
        let is_signed = dtype.is_int_signed();

        Self::Int {
            value,
            width,
            is_signed,
        }
    }

    #[inline]
    pub fn float(value: f64, dtype: Dtype) -> Self {
        let width = dtype
            .get_float_width()
            .expect("`dtype` must be `Dtype::Float`");

        Self::Float {
            value: value.into(),
            width,
        }
    }

    #[inline]
    pub fn global_variable(name: String, dtype: Dtype) -> Self {
        Self::GlobalVariable { name, dtype }
    }

    #[inline]
    pub fn get_int(&self) -> Option<(u128, usize, bool)> {
        if let Self::Int {
            value,
            width,
            is_signed,
        } = self
        {
            Some((*value, *width, *is_signed))
        } else {
            None
        }
    }

    #[inline]
    pub fn get_global_variable_name(&self) -> Option<String> {
        if let Self::GlobalVariable { name, .. } = self {
            Some(name.clone())
        } else {
            None
        }
    }

    #[inline]
    fn minus(self) -> Self {
        match self {
            Self::Int {
                value,
                width,
                is_signed,
            } => {
                assert!(is_signed);
                let minus_value = -(value as i128);
                Self::Int {
                    value: minus_value as u128,
                    width,
                    is_signed,
                }
            }
            Self::Float { mut value, width } => {
                *value.as_mut() *= -1.0f64;
                Self::Float { value, width }
            }
            _ => panic!(
                "constant value generated by `Constant::from_ast_expression` \
                 must be `Constant{{Int, Float}}`"
            ),
        }
    }

    pub fn is_undef(&self) -> bool {
        if let Self::Undef { .. } = self {
            true
        } else {
            false
        }
    }

    pub fn typecast(self, target_dtype: Dtype) -> Self {
        if self.dtype() == target_dtype {
            return self;
        }

        match (&self, &target_dtype) {
            (
                Constant::Int { value, width, .. },
                Dtype::Int {
                    width: target_width,
                    is_signed: target_signed,
                    ..
                },
            ) => {
                let result = if *target_signed {
                    if *width >= *target_width {
                        let value = trim_unnecessary_bits(*value, *target_width as u128);
                        sign_extension(value, *target_width as u128)
                    } else {
                        *value
                    }
                } else {
                    trim_unnecessary_bits(*value, *target_width as u128)
                };

                Constant::int(result, target_dtype)
            }
            (
                Constant::Int {
                    value, is_signed, ..
                },
                Dtype::Float { .. },
            ) => {
                let casted_value = if *is_signed {
                    *value as i128 as f64
                } else {
                    *value as f64
                };

                Constant::float(casted_value, target_dtype)
            }
            (Constant::Float { value, .. }, Dtype::Int { is_signed, .. }) => {
                let casted_value = if *is_signed {
                    value.into_inner() as i128 as u128
                } else {
                    value.into_inner() as u128
                };

                Constant::int(casted_value, target_dtype)
            }
            (Constant::Float { value, .. }, Dtype::Float { .. }) => {
                Constant::float(value.into_inner(), target_dtype)
            }
            _ => todo!("typecast ({:?}) {:?}", self, target_dtype),
        }
    }
}

#[inline]
pub fn sign_extension(value: u128, width: u128) -> u128 {
    let base = 1u128 << (width - 1);
    if value >= base {
        let bit_mask = -1i128 << (width as i128);
        value | bit_mask as u128
    } else {
        value
    }
}

#[inline]
pub fn trim_unnecessary_bits(value: u128, width: u128) -> u128 {
    let bit_mask = (1u128 << width) - 1;
    value & bit_mask
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Undef { .. } => write!(f, "undef"),
            Self::Unit => write!(f, "unit"),
            Self::Int {
                value, is_signed, ..
            } => write!(
                f,
                "{}",
                if *is_signed {
                    (*value as i128).to_string()
                } else {
                    value.to_string()
                }
            ),
            Self::Float { value, .. } => write!(f, "{}", value),
            Self::GlobalVariable { name, .. } => write!(f, "@{}", name),
        }
    }
}

impl HasDtype for Constant {
    fn dtype(&self) -> Dtype {
        match self {
            Self::Undef { dtype } => dtype.clone(),
            Self::Unit => Dtype::unit(),
            Self::Int {
                width, is_signed, ..
            } => Dtype::int(*width).set_signed(*is_signed),
            Self::Float { width, .. } => Dtype::float(*width),
            Self::GlobalVariable { dtype, .. } => Dtype::pointer(dtype.clone()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Named<T> {
    name: Option<String>,
    inner: T,
}

impl<T> Deref for Named<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<T> DerefMut for Named<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> Named<T> {
    pub fn new(name: Option<String>, inner: T) -> Self {
        Self { name, inner }
    }

    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }
}
