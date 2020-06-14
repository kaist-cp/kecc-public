use core::fmt;
use core::iter;
use core::mem;
use failure::Fail;
use ordered_float::OrderedFloat;
use std::collections::HashMap;

use itertools::izip;

use crate::ir::*;
use crate::*;

// TODO: delete `allow(dead_code)`
/// Even though `Undef`, `Int`, `Float` are constructed and actively used at run-time,
/// the rust compiler analyzes these elements are dead code.
/// For this reason, we add `allow(dead_code)` mark above these elements respectively.
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    #[allow(dead_code)]
    Undef {
        dtype: Dtype,
    },
    Unit,
    #[allow(dead_code)]
    Int {
        value: u128,
        width: usize,
        is_signed: bool,
    },
    #[allow(dead_code)]
    Float {
        /// `value` may be `f32`, but it is possible to consider it as `f64`.
        ///
        /// * Casting from an f32 to an f64 is perfect and lossless (f32 -> f64)
        /// * Casting from an f64 to an f32 will produce the closest possible value (f64 -> f32)
        /// https://doc.rust-lang.org/stable/reference/expressions/operator-expr.html#type-cast-expressions
        value: OrderedFloat<f64>,
        width: usize,
    },
    Pointer {
        bid: Option<usize>,
        offset: isize,
        dtype: Dtype,
    },
    Array {
        inner_dtype: Dtype,
        values: Vec<Value>,
    },
    Struct {
        name: String,
        fields: Vec<Named<Value>>,
    },
}

impl TryFrom<Constant> for Value {
    type Error = ();

    fn try_from(constant: Constant) -> Result<Self, Self::Error> {
        let value = match constant {
            Constant::Undef { dtype } => Self::Undef { dtype },
            Constant::Unit => Self::Unit,
            Constant::Int {
                value,
                width,
                is_signed,
            } => Self::Int {
                value,
                width,
                is_signed,
            },
            Constant::Float { value, width } => Self::Float { value, width },
            _ => panic!(),
        };

        Ok(value)
    }
}

impl HasDtype for Value {
    fn dtype(&self) -> Dtype {
        match self {
            Self::Undef { dtype } => dtype.clone(),
            Self::Unit => Dtype::unit(),
            Self::Int {
                width, is_signed, ..
            } => Dtype::int(*width).set_signed(*is_signed),
            Self::Float { width, .. } => Dtype::float(*width),
            Self::Pointer { dtype, .. } => Dtype::pointer(dtype.clone()),
            Self::Array {
                inner_dtype,
                values,
            } => Dtype::array(inner_dtype.clone(), values.len()),
            Self::Struct { name, fields } => {
                let fields = fields
                    .iter()
                    .map(|f| Named::new(f.name().cloned(), f.deref().dtype()))
                    .collect();
                Dtype::structure(Some(name.clone()), Some(fields))
            }
        }
    }
}

impl Value {
    #[inline]
    fn undef(dtype: Dtype) -> Self {
        Self::Undef { dtype }
    }

    #[inline]
    fn unit() -> Self {
        Self::Unit
    }

    #[inline]
    fn int(value: u128, width: usize, is_signed: bool) -> Self {
        Self::Int {
            value,
            width,
            is_signed,
        }
    }

    #[inline]
    fn float(value: f64, width: usize) -> Self {
        Self::Float {
            value: value.into(),
            width,
        }
    }

    #[inline]
    fn pointer(bid: Option<usize>, offset: isize, dtype: Dtype) -> Self {
        Self::Pointer { bid, offset, dtype }
    }

    #[inline]
    fn array(inner_dtype: Dtype, values: Vec<Self>) -> Self {
        Self::Array {
            inner_dtype,
            values,
        }
    }

    #[inline]
    fn structure(name: String, fields: Vec<Named<Value>>) -> Self {
        Self::Struct { name, fields }
    }

    #[inline]
    pub fn get_int(self) -> Option<(u128, usize, bool)> {
        if let Value::Int {
            value,
            width,
            is_signed,
        } = self
        {
            Some((value, width, is_signed))
        } else {
            None
        }
    }

    #[inline]
    fn get_pointer(&self) -> Option<(&Option<usize>, &isize, &Dtype)> {
        if let Value::Pointer { bid, offset, dtype } = self {
            Some((bid, offset, dtype))
        } else {
            None
        }
    }

    #[inline]
    fn nullptr(dtype: Dtype) -> Self {
        Self::Pointer {
            bid: None,
            offset: 0,
            dtype,
        }
    }

    #[inline]
    fn default_from_dtype(
        dtype: &Dtype,
        structs: &HashMap<String, Option<Dtype>>,
    ) -> Result<Self, ()> {
        let value = match dtype {
            Dtype::Unit { .. } => Self::unit(),
            Dtype::Int {
                width, is_signed, ..
            } => Self::int(u128::default(), *width, *is_signed),
            Dtype::Float { width, .. } => Self::float(f64::default(), *width),
            Dtype::Pointer { inner, .. } => Self::nullptr(inner.deref().clone()),
            Dtype::Array { inner, size } => {
                let values = iter::repeat(Self::default_from_dtype(inner, structs))
                    .take(*size)
                    .collect::<Result<Vec<_>, _>>()?;
                Self::array(inner.deref().clone(), values)
            }
            Dtype::Struct { name, .. } => {
                let name = name.as_ref().expect("struct should have its name");
                let struct_type = structs
                    .get(name)
                    .expect("struct type matched with `name` must exist")
                    .as_ref()
                    .expect("`struct_type` must have its definition");
                let fields = struct_type
                    .get_struct_fields()
                    .expect("`struct_type` must be struct type")
                    .as_ref()
                    .expect("`fields` must be `Some`");

                let fields = fields
                    .iter()
                    .map(|f| {
                        let value = Self::default_from_dtype(f.deref(), structs)?;
                        Ok(Named::new(f.name().cloned(), value))
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                Self::structure(name.clone(), fields)
            }
            Dtype::Function { .. } => panic!("function type does not have a default value"),
            Dtype::Typedef { .. } => panic!("typedef should be replaced by real dtype"),
        };

        Ok(value)
    }

    fn try_from_initializer(
        initializer: &ast::Initializer,
        dtype: &Dtype,
        structs: &HashMap<String, Option<Dtype>>,
    ) -> Result<Self, ()> {
        match initializer {
            ast::Initializer::Expression(expr) => match dtype {
                Dtype::Int { .. } | Dtype::Float { .. } | Dtype::Pointer { .. } => {
                    let constant = Constant::try_from(&expr.node)?;
                    let value = Self::try_from(constant)?;

                    calculator::calculate_typecast(value, dtype.clone())
                }
                _ => Err(()),
            },
            ast::Initializer::List(items) => match dtype {
                Dtype::Array { inner, size } => {
                    let inner_dtype = inner.deref().clone();
                    let num_of_items = items.len();
                    let values = (0..*size)
                        .map(|i| {
                            if i < num_of_items {
                                Self::try_from_initializer(
                                    &items[i].node.initializer.node,
                                    &inner_dtype,
                                    structs,
                                )
                            } else {
                                Self::default_from_dtype(&inner_dtype, structs)
                            }
                        })
                        .collect::<Result<Vec<_>, _>>()?;

                    Ok(Self::array(inner_dtype, values))
                }
                Dtype::Struct { name, .. } => {
                    let name = name.as_ref().expect("struct should have its name");
                    let struct_type = structs
                        .get(name)
                        .expect("struct type matched with `name` must exist")
                        .as_ref()
                        .expect("`struct_type` must have its definition");
                    let fields = struct_type
                        .get_struct_fields()
                        .expect("`struct_type` must be struct type")
                        .as_ref()
                        .expect("`fields` must be `Some`");

                    let fields = fields
                        .iter()
                        .enumerate()
                        .map(|(i, f)| {
                            let value = if let Some(item) = items.get(i) {
                                Self::try_from_initializer(
                                    &item.node.initializer.node,
                                    f.deref(),
                                    structs,
                                )?
                            } else {
                                Self::default_from_dtype(f.deref(), structs)?
                            };

                            Ok(Named::new(f.name().cloned(), value))
                        })
                        .collect::<Result<Vec<_>, _>>()?;

                    Ok(Self::structure(name.clone(), fields))
                }
                _ => Err(()),
            },
        }
    }
}

#[derive(Debug, PartialEq, Fail)]
pub enum InterpreterError {
    #[fail(display = "current block is unreachable")]
    Unreachable,
    #[fail(display = "ir has no main function")]
    NoMainFunction,
    #[fail(display = "ir has no function definition of {} function", func_name)]
    NoFunctionDefinition { func_name: String },
    #[fail(
        display = "ir has no structure definition of {} structure",
        struct_name
    )]
    NoStructureDefinition { struct_name: String },
    #[fail(display = "{}:{} / {}", func_name, pc, msg)]
    Misc {
        func_name: String,
        pc: Pc,
        msg: String,
    },
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Pc {
    pub bid: BlockId,
    pub iid: usize,
}

impl fmt::Display for Pc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.bid, self.iid)
    }
}

impl Pc {
    fn new(bid: BlockId) -> Pc {
        Pc { bid, iid: 0 }
    }

    fn increment(&mut self) {
        self.iid += 1;
    }
}

#[derive(Default, Debug, PartialEq, Clone)]
struct RegisterMap {
    inner: HashMap<RegisterId, Value>,
}

impl RegisterMap {
    fn read(&self, rid: RegisterId) -> &Value {
        self.inner
            .get(&rid)
            .expect("`rid` must be assigned before it can be used")
    }

    fn write(&mut self, rid: RegisterId, value: Value) {
        let _ = self.inner.insert(rid, value);
    }
}

#[derive(Default, Debug, PartialEq, Clone)]
/// Bidirectional map between the name of a global variable and memory box id
struct GlobalMap {
    /// Map name of a global variable to memory box id
    ///
    /// Since IR treats global variable as `Constant::GlobalVariable`,
    /// the interpreter should be able to generate pointer values by infer 'bid'
    /// from the 'name' of the global variable.
    var_to_bid: HashMap<String, usize>,
    /// Map memory box id to the name of a global variable
    ///
    /// When a function call occurs, the interpreter should be able to find `name` of the function
    /// from `bid` of the `callee` which is a function pointer.
    bid_to_var: HashMap<usize, String>,
}

impl GlobalMap {
    /// Create a bi-directional mapping between `var` and `bid`.
    fn insert(&mut self, var: String, bid: usize) -> Result<(), InterpreterError> {
        if self.var_to_bid.insert(var.clone(), bid).is_some() {
            panic!("variable name should be unique in IR")
        }
        if self.bid_to_var.insert(bid, var).is_some() {
            panic!("`bid` is connected to only one `var`")
        }

        Ok(())
    }

    fn get_bid(&self, var: &str) -> Option<usize> {
        self.var_to_bid.get(var).cloned()
    }

    fn get_var(&self, bid: usize) -> Option<String> {
        self.bid_to_var.get(&bid).cloned()
    }
}

#[derive(Debug, PartialEq, Clone)]
struct StackFrame<'i> {
    pub pc: Pc,
    pub registers: RegisterMap,
    pub func_name: String,
    pub func_def: &'i FunctionDefinition,
}

impl<'i> StackFrame<'i> {
    fn new(bid: BlockId, func_name: String, func_def: &'i FunctionDefinition) -> Self {
        StackFrame {
            pc: Pc::new(bid),
            registers: Default::default(),
            func_name,
            func_def,
        }
    }
}

mod calculator {
    use super::Value;
    use crate::ir::*;
    use lang_c::ast;
    use std::cmp::Ordering;

    fn calculate_integer_binary_operator_expression(
        op: &ast::BinaryOperator,
        lhs: u128,
        rhs: u128,
        width: usize,
        is_signed: bool,
    ) -> Result<Value, ()> {
        let result = match op {
            // TODO: explain why plus & minus do not need to consider `is_signed'
            ast::BinaryOperator::Plus => (lhs as i128 + rhs as i128) as u128,
            ast::BinaryOperator::Minus => (lhs as i128 - rhs as i128) as u128,
            ast::BinaryOperator::Multiply => {
                if is_signed {
                    (lhs as i128 * rhs as i128) as u128
                } else {
                    lhs * rhs
                }
            }
            ast::BinaryOperator::Divide => {
                assert!(rhs != 0);
                if is_signed {
                    (lhs as i128 / rhs as i128) as u128
                } else {
                    lhs / rhs
                }
            }
            ast::BinaryOperator::Modulo => {
                assert!(rhs != 0);
                if is_signed {
                    (lhs as i128 % rhs as i128) as u128
                } else {
                    lhs % rhs
                }
            }
            ast::BinaryOperator::ShiftLeft => {
                let rhs = if is_signed {
                    let rhs = rhs as i128;
                    assert!(rhs >= 0);
                    assert!(rhs < (width as i128));
                    rhs as u128
                } else {
                    assert!(rhs < (width as u128));
                    rhs
                };

                lhs << rhs
            }
            ast::BinaryOperator::ShiftRight => {
                if is_signed {
                    // arithmetic shift right
                    let rhs = rhs as i128;
                    assert!(rhs >= 0);
                    assert!(rhs < (width as i128));
                    ((lhs as i128) >> rhs) as u128
                } else {
                    // logical shift right
                    assert!(rhs < (width as u128));
                    let bit_mask = (1u128 << width as u128) - 1;
                    let lhs = lhs & bit_mask;
                    lhs >> rhs
                }
            }
            ast::BinaryOperator::BitwiseAnd => lhs & rhs,
            ast::BinaryOperator::BitwiseXor => lhs ^ rhs,
            ast::BinaryOperator::BitwiseOr => lhs | rhs,
            ast::BinaryOperator::Equals => {
                let result = if lhs == rhs { 1 } else { 0 };
                return Ok(Value::int(result, 1, false));
            }
            ast::BinaryOperator::NotEquals => {
                let result = if lhs != rhs { 1 } else { 0 };
                return Ok(Value::int(result, 1, false));
            }
            ast::BinaryOperator::Less => {
                let condition = if is_signed {
                    (lhs as i128) < (rhs as i128)
                } else {
                    lhs < rhs
                };
                let result = if condition { 1 } else { 0 };
                return Ok(Value::int(result, 1, false));
            }
            ast::BinaryOperator::Greater => {
                let condition = if is_signed {
                    (lhs as i128) > (rhs as i128)
                } else {
                    lhs > rhs
                };
                let result = if condition { 1 } else { 0 };
                return Ok(Value::int(result, 1, false));
            }
            ast::BinaryOperator::LessOrEqual => {
                let condition = if is_signed {
                    (lhs as i128) <= (rhs as i128)
                } else {
                    lhs <= rhs
                };
                let result = if condition { 1 } else { 0 };
                return Ok(Value::int(result, 1, false));
            }
            ast::BinaryOperator::GreaterOrEqual => {
                let condition = if is_signed {
                    (lhs as i128) >= (rhs as i128)
                } else {
                    lhs >= rhs
                };
                let result = if condition { 1 } else { 0 };
                return Ok(Value::int(result, 1, false));
            }
            _ => todo!(
                "calculate_binary_operator_expression: not supported operator {:?}",
                op
            ),
        };

        let result = if is_signed {
            sign_extension(result, width as u128)
        } else {
            trim_unnecessary_bits(result, width as u128)
        };

        Ok(Value::int(result, width, is_signed))
    }

    fn calculate_float_binary_operator_expression(
        op: &ast::BinaryOperator,
        lhs: OrderedFloat<f64>,
        rhs: OrderedFloat<f64>,
        width: usize,
    ) -> Result<Value, ()> {
        let result = match op {
            ast::BinaryOperator::Plus => lhs.into_inner() + rhs.into_inner(),
            ast::BinaryOperator::Minus => lhs.into_inner() - rhs.into_inner(),
            ast::BinaryOperator::Multiply => lhs.into_inner() * rhs.into_inner(),
            ast::BinaryOperator::Divide => {
                assert!(rhs.into_inner() != 0.0);
                lhs.into_inner() / rhs.into_inner()
            }
            ast::BinaryOperator::Equals => {
                let order = lhs
                    .partial_cmp(&rhs)
                    .expect("`lhs` and `rhs` must be not NAN");
                let result = if Ordering::Equal == order { 1 } else { 0 };
                return Ok(Value::int(result, 1, false));
            }
            ast::BinaryOperator::NotEquals => {
                let order = lhs
                    .partial_cmp(&rhs)
                    .expect("`lhs` and `rhs` must be not NAN");
                let result = if Ordering::Equal != order { 1 } else { 0 };
                return Ok(Value::int(result, 1, false));
            }
            ast::BinaryOperator::Less => {
                let result = if lhs.lt(&rhs) { 1 } else { 0 };
                return Ok(Value::int(result, 1, false));
            }
            ast::BinaryOperator::Greater => {
                let result = if lhs.gt(&rhs) { 1 } else { 0 };
                return Ok(Value::int(result, 1, false));
            }
            ast::BinaryOperator::LessOrEqual => {
                let result = if lhs.le(&rhs) { 1 } else { 0 };
                return Ok(Value::int(result, 1, false));
            }
            ast::BinaryOperator::GreaterOrEqual => {
                let result = if lhs.ge(&rhs) { 1 } else { 0 };
                return Ok(Value::int(result, 1, false));
            }
            _ => todo!(
                "calculate_binary_operator_expression: not supported case for \
                 {:?} {:?} {:?}",
                op,
                lhs,
                rhs
            ),
        };

        Ok(Value::float(result, width))
    }

    // TODO: change to template function in the future
    pub fn calculate_binary_operator_expression(
        op: &ast::BinaryOperator,
        lhs: Value,
        rhs: Value,
    ) -> Result<Value, ()> {
        match (lhs, rhs) {
            (Value::Undef { dtype }, _) => Ok(Value::undef(dtype)),
            (_, Value::Undef { dtype }) => Ok(Value::undef(dtype)),
            (
                Value::Int {
                    value: lhs,
                    width: lhs_w,
                    is_signed: lhs_s,
                },
                Value::Int {
                    value: rhs,
                    width: rhs_w,
                    is_signed: rhs_s,
                },
            ) => {
                assert_eq!(lhs_w, rhs_w);
                assert_eq!(lhs_s, rhs_s);

                calculate_integer_binary_operator_expression(op, lhs, rhs, lhs_w, lhs_s)
            }
            (
                Value::Float {
                    value: lhs,
                    width: lhs_w,
                },
                Value::Float {
                    value: rhs,
                    width: rhs_w,
                },
            ) => {
                assert_eq!(lhs_w, rhs_w);

                calculate_float_binary_operator_expression(op, lhs, rhs, lhs_w)
            }
            (
                Value::Pointer { bid, offset, .. },
                Value::Pointer {
                    bid: other_bid,
                    offset: other_offset,
                    ..
                },
            ) => match op {
                ast::BinaryOperator::Equals => {
                    let result = if bid == other_bid && offset == other_offset {
                        1
                    } else {
                        0
                    };
                    Ok(Value::int(result, 1, false))
                }
                ast::BinaryOperator::NotEquals => {
                    let result = if !(bid == other_bid && offset == other_offset) {
                        1
                    } else {
                        0
                    };
                    Ok(Value::int(result, 1, false))
                }
                _ => todo!(
                    "calculate_binary_operator_expression: not supported case for \
                     {:?} between pointer and integer value",
                    op,
                ),
            },
            (lhs, rhs) => todo!(
                "calculate_binary_operator_expression: not supported case for {:?} {:?} {:?}",
                op,
                lhs,
                rhs
            ),
        }
    }

    pub fn calculate_unary_operator_expression(
        op: &ast::UnaryOperator,
        operand: Value,
    ) -> Result<Value, ()> {
        match operand {
            Value::Undef { dtype } => Ok(Value::undef(dtype)),
            Value::Int {
                value,
                width,
                is_signed,
            } => {
                match op {
                    ast::UnaryOperator::Plus => Ok(Value::int(value, width, is_signed)),
                    ast::UnaryOperator::Minus => {
                        let result = if is_signed {
                            (-(value as i128)) as u128
                        } else {
                            let value = (-(value as i128)) as u128;
                            trim_unnecessary_bits(value, width as u128)
                        };
                        Ok(Value::int(result as u128, width, is_signed))
                    }
                    ast::UnaryOperator::Negate => {
                        // Check if it is boolean
                        assert!(width == 1);
                        let result = if value == 0 { 1 } else { 0 };
                        Ok(Value::int(result, width, is_signed))
                    }
                    _ => todo!(
                        "calculate_unary_operator_expression: not supported case for {:?} {:?}",
                        op,
                        operand,
                    ),
                }
            }
            Value::Float { value, width } => match op {
                ast::UnaryOperator::Plus => Ok(Value::float(value.into_inner(), width)),
                ast::UnaryOperator::Minus => Ok(Value::float(-value.into_inner(), width)),
                _ => todo!(
                    "calculate_unary_operator_expression: not supported case for {:?} {:?}",
                    op,
                    operand,
                ),
            },
            _ => todo!(
                "calculate_unary_operator_expression: not supported case for {:?} {:?}",
                op,
                operand,
            ),
        }
    }

    pub fn calculate_typecast(value: Value, dtype: Dtype) -> Result<Value, ()> {
        if value.dtype() == dtype {
            return Ok(value);
        }

        match (value, dtype) {
            (Value::Undef { .. }, dtype) => Ok(Value::undef(dtype)),
            (
                Value::Int { value, width, .. },
                Dtype::Int {
                    width: target_width,
                    is_signed: target_signed,
                    ..
                },
            ) => {
                let result = if target_signed {
                    if width >= target_width {
                        // TODO: explain the logic in the future
                        let value = trim_unnecessary_bits(value, target_width as u128);
                        sign_extension(value, target_width as u128)
                    } else {
                        value
                    }
                } else {
                    trim_unnecessary_bits(value, target_width as u128)
                };

                Ok(Value::int(result, target_width, target_signed))
            }
            (
                Value::Int {
                    value, is_signed, ..
                },
                Dtype::Float { width, .. },
            ) => {
                let casted_value = if is_signed {
                    value as i128 as f64
                } else {
                    value as f64
                };
                Ok(Value::float(casted_value, width))
            }
            (Value::Int { value, .. }, Dtype::Pointer { inner, .. }) => {
                if value == 0 {
                    Ok(Value::pointer(None, 0, inner.deref().clone()))
                } else {
                    panic!(format!(
                        "calculate_typecast: not support case \
                         typecast int to pointer when `value` is {}",
                        value
                    ))
                }
            }
            (
                Value::Float { value, .. },
                Dtype::Int {
                    width, is_signed, ..
                },
            ) => {
                let casted_value = if is_signed {
                    value.into_inner() as i128 as u128
                } else {
                    value.into_inner() as u128
                };
                Ok(Value::int(casted_value, width, is_signed))
            }
            (Value::Float { value, .. }, Dtype::Float { width, .. }) => {
                Ok(Value::float(value.into_inner(), width))
            }
            (value, dtype) => todo!("calculate_typecast ({:?}) {:?}", value, dtype),
        }
    }
}

// TODO: delete `allow(dead_code)`
/// Even though `Pointer` variant is constructed and actively used at run-time,
/// the rust compiler analyzes it is dead code.
/// For this reason, we add `allow(dead_code)` mark.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum Byte {
    Undef,
    Concrete(u8),
    #[allow(dead_code)]
    Pointer {
        bid: Option<usize>,
        offset: isize,
        index: usize,
    },
}

#[derive(Default, Debug, PartialEq)]
struct Memory {
    inner: Vec<Option<Vec<Byte>>>,
}

impl Byte {
    #[inline]
    fn concrete(byte: u8) -> Self {
        Self::Concrete(byte)
    }

    #[inline]
    fn pointer(bid: Option<usize>, offset: isize, index: usize) -> Self {
        Self::Pointer { bid, offset, index }
    }

    fn get_concrete(&self) -> Option<u8> {
        if let Self::Concrete(byte) = self {
            Some(*byte)
        } else {
            None
        }
    }

    fn get_pointer(&self) -> Option<(Option<usize>, isize, usize)> {
        if let Self::Pointer { bid, offset, index } = self {
            Some((*bid, *offset, *index))
        } else {
            None
        }
    }

    fn block_from_dtype(dtype: &Dtype, structs: &HashMap<String, Option<Dtype>>) -> Vec<Self> {
        let size = dtype.size_align_of(structs).unwrap().0;
        iter::repeat(Self::Undef).take(size).collect()
    }

    fn u128_to_bytes(mut value: u128, size: usize) -> Vec<u8> {
        let divisor = 1u128 << Dtype::BITS_OF_BYTE;
        let mut bytes = Vec::new();
        for _ in 0..size {
            bytes.push((value % divisor) as u8);
            value /= divisor;
        }

        bytes
    }

    fn bytes_to_u128(bytes: &[u8], is_signed: bool) -> u128 {
        let width = bytes.len();
        assert!(0 < width && width <= 16);

        let is_negative = is_signed && *bytes.last().unwrap() >= 128;
        let mut array = [if is_negative { 255 } else { 0 }; 16];
        array[0..width].copy_from_slice(bytes);
        u128::from_le_bytes(array)
    }

    fn bytes_to_value<'b, I>(
        bytes: &mut I,
        dtype: &Dtype,
        structs: &HashMap<String, Option<Dtype>>,
    ) -> Result<Value, InterpreterError>
    where
        I: Iterator<Item = &'b Self>,
    {
        match dtype {
            Dtype::Unit { .. } => Ok(Value::Unit),
            Dtype::Int {
                width, is_signed, ..
            } => {
                let size = dtype.size_align_of(structs).unwrap().0;
                let bytes = bytes.by_ref().take(size).collect::<Vec<_>>();
                let value = some_or!(
                    bytes
                        .iter()
                        .map(|b| b.get_concrete())
                        .collect::<Option<Vec<_>>>(),
                    return Ok(Value::undef(dtype.clone()))
                );
                let value = Self::bytes_to_u128(&value, *is_signed);
                Ok(Value::int(value, *width, *is_signed))
            }
            Dtype::Float { width, .. } => {
                let size = dtype.size_align_of(structs).unwrap().0;
                let bytes = bytes.by_ref().take(size).collect::<Vec<_>>();
                let value = some_or!(
                    bytes
                        .iter()
                        .map(|b| b.get_concrete())
                        .collect::<Option<Vec<_>>>(),
                    return Ok(Value::undef(dtype.clone()))
                );
                let value = Self::bytes_to_u128(&value, false);
                let value = if size == Dtype::SIZE_OF_FLOAT {
                    f32::from_bits(value as u32) as f64
                } else {
                    f64::from_bits(value as u64)
                };

                Ok(Value::float(value, *width))
            }
            Dtype::Pointer { inner, .. } => {
                let bytes = bytes
                    .by_ref()
                    .take(Dtype::SIZE_OF_POINTER)
                    .collect::<Vec<_>>();
                let value = some_or!(
                    bytes
                        .iter()
                        .map(|b| b.get_pointer())
                        .collect::<Option<Vec<_>>>(),
                    return Ok(Value::undef(dtype.clone()))
                );

                let (bid, offset, _) = value.first().expect("not empty");

                Ok(
                    if !value
                        .iter()
                        .enumerate()
                        .all(|(idx, ptr)| *ptr == (*bid, *offset, idx))
                    {
                        Value::undef(inner.deref().clone())
                    } else {
                        Value::pointer(*bid, *offset, inner.deref().clone())
                    },
                )
            }
            Dtype::Array { inner, size } => {
                let (inner_size, inner_align) = inner.size_align_of(structs).unwrap();
                let padding = std::cmp::max(inner_size, inner_align) - inner_size;
                let values = (0..*size)
                    .map(|_| {
                        let value = Self::bytes_to_value(bytes, inner, structs)?;
                        if padding > 0 {
                            let _ = bytes.by_ref().nth(padding - 1);
                        }
                        Ok(value)
                    })
                    .collect::<Result<Vec<_>, InterpreterError>>()?;
                Ok(Value::Array {
                    inner_dtype: inner.deref().clone(),
                    values,
                })
            }
            Dtype::Struct { name, .. } => {
                let name = name.as_ref().expect("struct should have its name");
                let struct_type = structs
                    .get(name)
                    .expect("struct type matched with `name` must exist")
                    .as_ref()
                    .expect("`struct_type` must have its definition");
                let fields = struct_type
                    .get_struct_fields()
                    .expect("`struct_type` must be struct type")
                    .as_ref()
                    .expect("`fields` must be `Some`");
                let (size, _, offsets) = struct_type
                    .get_struct_size_align_offsets()
                    .expect("`struct_type` must be struct type")
                    .as_ref()
                    .expect("`offsets` must be `Some`");
                let bytes = bytes.by_ref().take(*size).cloned().collect::<Vec<_>>();

                assert_eq!(fields.len(), offsets.len());
                let fields = izip!(fields, offsets)
                    .map(|(f, o)| {
                        let mut sub_bytes = bytes[*o..].iter();
                        let value = Self::bytes_to_value(&mut sub_bytes, f.deref(), structs)?;
                        Ok(Named::new(f.name().cloned(), value))
                    })
                    .collect::<Result<Vec<_>, InterpreterError>>()?;

                Ok(Value::Struct {
                    name: name.clone(),
                    fields,
                })
            }
            Dtype::Function { .. } => panic!("function value cannot be constructed from bytes"),
            Dtype::Typedef { .. } => panic!("typedef should be replaced by real dtype"),
        }
    }

    fn value_to_bytes(value: &Value, structs: &HashMap<String, Option<Dtype>>) -> Vec<Self> {
        match value {
            Value::Undef { dtype } => Self::block_from_dtype(dtype, structs),
            Value::Unit => Vec::new(),
            Value::Int {
                value: int_value, ..
            } => {
                let size = value.dtype().size_align_of(structs).unwrap().0;
                Self::u128_to_bytes(*int_value, size)
                    .iter()
                    .map(|b| Self::concrete(*b))
                    .collect::<Vec<_>>()
            }
            Value::Float {
                value: float_value, ..
            } => {
                let size = value.dtype().size_align_of(structs).unwrap().0;
                let value_bits: u128 = match size {
                    Dtype::SIZE_OF_FLOAT => (float_value.into_inner() as f32).to_bits() as u128,
                    Dtype::SIZE_OF_DOUBLE => (float_value.into_inner() as f64).to_bits() as u128,
                    _ => panic!("value_to_bytes: {} is not a valid float size", size),
                };

                Self::u128_to_bytes(value_bits, size)
                    .iter()
                    .map(|b| Self::concrete(*b))
                    .collect::<Vec<_>>()
            }
            Value::Pointer { bid, offset, .. } => (0..Dtype::SIZE_OF_POINTER)
                .map(|i| Self::pointer(*bid, *offset, i))
                .collect(),
            Value::Array {
                inner_dtype,
                values,
            } => {
                let (inner_size, inner_align) = inner_dtype.size_align_of(structs).unwrap();
                let padding = std::cmp::max(inner_size, inner_align) - inner_size;
                values
                    .iter()
                    .map(|v| {
                        let mut result = Self::value_to_bytes(v, structs);
                        result.extend(iter::repeat(Byte::Undef).take(padding));
                        result
                    })
                    .flatten()
                    .collect()
            }
            Value::Struct { name, fields } => {
                let struct_type = structs
                    .get(name)
                    .expect("struct type matched with `name` must exist")
                    .as_ref()
                    .expect("`struct_type` must have its definition");
                let (size_of, _, offsets) = struct_type
                    .get_struct_size_align_offsets()
                    .expect("`struct_type` must be struct type")
                    .as_ref()
                    .expect("`offsets` must be `Some`");
                let mut values = iter::repeat(Byte::Undef).take(*size_of).collect::<Vec<_>>();

                assert_eq!(fields.len(), offsets.len());
                izip!(fields, offsets).for_each(|(f, o)| {
                    let result = Self::value_to_bytes(f.deref(), structs);
                    let size_of_data = f.deref().dtype().size_align_of(structs).unwrap().0;
                    values.splice(*o..(*o + size_of_data), result.iter().cloned());
                });

                values
            }
        }
    }
}

impl Memory {
    fn alloc(
        &mut self,
        dtype: &Dtype,
        structs: &HashMap<String, Option<Dtype>>,
    ) -> Result<usize, InterpreterError> {
        let bid = self.inner.len();
        self.inner
            .push(Some(Byte::block_from_dtype(dtype, structs)));
        Ok(bid)
    }

    fn dealloc(
        &mut self,
        bid: usize,
        offset: isize,
        dtype: &Dtype,
        structs: &HashMap<String, Option<Dtype>>,
    ) -> Result<(), InterpreterError> {
        let block = &mut self.inner[bid];
        assert_eq!(offset, 0);
        assert_eq!(
            block.as_mut().unwrap().len(),
            dtype.size_align_of(structs).unwrap().0
        );
        *block = None;
        Ok(())
    }

    fn load(
        &self,
        bid: usize,
        offset: isize,
        dtype: &Dtype,
        structs: &HashMap<String, Option<Dtype>>,
    ) -> Result<Value, InterpreterError> {
        let size = dtype.size_align_of(structs).unwrap().0;
        let end = offset as usize + size;
        let block = self.inner[bid].as_ref().unwrap();

        if 0 <= offset && end <= block.len() {
            let mut iter = block[offset as usize..end].iter();
            Byte::bytes_to_value(&mut iter, dtype, structs)
        } else {
            Ok(Value::undef(dtype.clone()))
        }
    }

    fn store(
        &mut self,
        bid: usize,
        offset: isize,
        value: &Value,
        structs: &HashMap<String, Option<Dtype>>,
    ) -> Result<(), ()> {
        let size = value.dtype().size_align_of(structs).unwrap().0;
        let end = offset as usize + size;
        let bytes = Byte::value_to_bytes(value, structs);
        let block = self.inner[bid].as_mut().unwrap();

        if 0 <= offset && end <= block.len() {
            block.splice(offset as usize..end, bytes.iter().cloned());
            Ok(())
        } else {
            Err(())
        }
    }
}

#[derive(Debug, PartialEq)]
struct State<'i> {
    /// A data structure that maps each global variable to a pointer value
    /// When function call occurs, `registers` can be initialized by `global_registers`
    pub global_map: GlobalMap,
    pub stack_frame: StackFrame<'i>,
    pub stack: Vec<StackFrame<'i>>,
    pub memory: Memory,
    pub ir: &'i TranslationUnit,
}

impl<'i> State<'i> {
    fn new(ir: &'i TranslationUnit, args: Vec<Value>) -> Result<State, InterpreterError> {
        // Interpreter starts with the main function
        let func_name = String::from("main");
        let func = ir
            .decls
            .get(&func_name)
            .ok_or_else(|| InterpreterError::NoMainFunction)?;
        let (_, func_def) = func
            .get_function()
            .ok_or_else(|| InterpreterError::NoMainFunction)?;
        let func_def = func_def
            .as_ref()
            .ok_or_else(|| InterpreterError::NoFunctionDefinition {
                func_name: func_name.clone(),
            })?;

        // Create State
        let mut state = State {
            global_map: GlobalMap::default(),
            stack_frame: StackFrame::new(func_def.bid_init, func_name, func_def),
            stack: Vec::new(),
            memory: Default::default(),
            ir,
        };

        state.alloc_global_variables()?;

        // Initialize state with main function and args
        state.write_args(func_def.bid_init, args)?;
        state.alloc_local_variables()?;

        Ok(state)
    }

    fn alloc_global_variables(&mut self) -> Result<(), InterpreterError> {
        for (name, decl) in &self.ir.decls {
            // Memory allocation
            let bid = self.memory.alloc(&decl.dtype(), &self.ir.structs)?;
            self.global_map.insert(name.clone(), bid)?;

            // Initialize allocated memory space
            match decl {
                Declaration::Variable { dtype, initializer } => {
                    let value = if let Some(initializer) = initializer {
                        Value::try_from_initializer(initializer, dtype, &self.ir.structs).map_err(
                            |_| InterpreterError::Misc {
                                func_name: self.stack_frame.func_name.clone(),
                                pc: self.stack_frame.pc,
                                msg: format!(
                                    "fail to translate `Initializer` and `{}` to `Value`",
                                    dtype
                                ),
                            },
                        )?
                    } else {
                        Value::default_from_dtype(&dtype, &self.ir.structs)
                            .expect("default value must be derived from `dtype`")
                    };

                    self.memory
                        .store(bid, 0, &value, &self.ir.structs)
                        .map_err(|_| InterpreterError::Misc {
                            func_name: self.stack_frame.func_name.clone(),
                            pc: self.stack_frame.pc,
                            msg: format!(
                                "fail to store {:?} into memory with bid: {}, offset: {}",
                                value, bid, 0,
                            ),
                        })?
                }
                // If functin declaration, skip initialization
                Declaration::Function { .. } => (),
            }
        }

        Ok(())
    }

    fn alloc_local_variables(&mut self) -> Result<(), InterpreterError> {
        // add alloc register
        for (id, allocation) in self.stack_frame.func_def.allocations.iter().enumerate() {
            let bid = self.memory.alloc(&allocation, &self.ir.structs)?;
            let ptr = Value::pointer(Some(bid), 0, allocation.deref().clone());
            let rid = RegisterId::local(id);

            self.stack_frame.registers.write(rid, ptr)
        }

        Ok(())
    }

    fn write_args(&mut self, bid_init: BlockId, args: Vec<Value>) -> Result<(), InterpreterError> {
        for (i, value) in args.iter().enumerate() {
            self.stack_frame
                .registers
                .write(RegisterId::arg(bid_init, i), value.clone());
        }

        Ok(())
    }

    fn step(&mut self) -> Result<Option<Value>, InterpreterError> {
        let block = self
            .stack_frame
            .func_def
            .blocks
            .get(&self.stack_frame.pc.bid)
            .expect("block matched with `bid` must be exist");

        // If it's time to execute an instruction, do so.
        if let Some(instr) = block.instructions.get(self.stack_frame.pc.iid) {
            self.interp_instruction(instr)?;
            return Ok(None);
        }

        // Execute a block exit.
        let return_value = some_or!(self.interp_block_exit(&block.exit)?, return Ok(None));

        // If it's returning from a function, pop the stack frame.

        // Frees memory allocated in the callee
        for (i, d) in self.stack_frame.func_def.allocations.iter().enumerate() {
            let (bid, offset, dtype) = self
                .stack_frame
                .registers
                .read(RegisterId::local(i))
                .get_pointer()
                .unwrap();
            assert_eq!(d.deref(), dtype);
            self.memory
                .dealloc(bid.unwrap(), *offset, dtype, &self.ir.structs)?;
        }

        // restore previous state
        let prev_stack_frame = some_or!(self.stack.pop(), return Ok(Some(return_value)));
        self.stack_frame = prev_stack_frame;

        // create temporary register to write return value
        let register = RegisterId::temp(self.stack_frame.pc.bid, self.stack_frame.pc.iid);
        self.stack_frame.registers.write(register, return_value);
        self.stack_frame.pc.increment();
        Ok(None)
    }

    fn run(&mut self) -> Result<Value, InterpreterError> {
        loop {
            if let Some(value) = self.step()? {
                return Ok(value);
            }
        }
    }

    fn interp_args(
        &self,
        signature: &FunctionSignature,
        args: &[Operand],
    ) -> Result<Vec<Value>, InterpreterError> {
        // Check that the dtype of each args matches the expected
        if !(args.len() == signature.params.len()
            && izip!(args, &signature.params)
                .all(|(a, d)| a.dtype().set_const(false) == d.clone().set_const(false)))
        {
            panic!("dtype of args and params must be compatible")
        }

        args.iter()
            .map(|a| self.interp_operand(a.clone()))
            .collect::<Result<Vec<_>, _>>()
    }

    fn interp_jump(&mut self, arg: &JumpArg) -> Result<Option<Value>, InterpreterError> {
        let block = self
            .stack_frame
            .func_def
            .blocks
            .get(&arg.bid)
            .expect("block matched with `arg.bid` must be exist");

        assert_eq!(arg.args.len(), block.phinodes.len());
        for (a, d) in izip!(&arg.args, &block.phinodes) {
            assert!(a.dtype().set_const(false) == d.deref().clone().set_const(false));
        }

        arg.args
            .iter()
            .map(|a| self.interp_operand(a.clone()).unwrap())
            .collect::<Vec<_>>()
            .into_iter()
            .enumerate()
            .for_each(|(i, v)| {
                self.stack_frame
                    .registers
                    .write(RegisterId::arg(arg.bid, i), v);
            });

        self.stack_frame.pc = Pc::new(arg.bid);
        Ok(None)
    }

    fn interp_block_exit(
        &mut self,
        block_exit: &BlockExit,
    ) -> Result<Option<Value>, InterpreterError> {
        match block_exit {
            BlockExit::Jump { arg } => self.interp_jump(arg),
            BlockExit::ConditionalJump {
                condition,
                arg_then,
                arg_else,
            } => {
                let value = self.interp_operand(condition.clone())?;
                let (value, width, _) = value.get_int().expect("`condition` must be `Value::Int`");
                // Check if it is boolean
                assert!(width == 1);

                self.interp_jump(if value == 1 { arg_then } else { arg_else })
            }
            BlockExit::Switch {
                value,
                default,
                cases,
            } => {
                let value = self.interp_operand(value.clone())?;

                // TODO: consider different integer `width` in the future
                let arg = cases
                    .iter()
                    .find(|(c, _)| value == self.interp_constant(c.clone()))
                    .map(|(_, arg)| arg)
                    .unwrap_or_else(|| default);
                self.interp_jump(arg)
            }
            BlockExit::Return { value } => Ok(Some(self.interp_operand(value.clone())?)),
            BlockExit::Unreachable => Err(InterpreterError::Unreachable),
        }
    }

    fn interp_instruction(&mut self, instruction: &Instruction) -> Result<(), InterpreterError> {
        let result = match instruction {
            Instruction::Nop => Value::unit(),
            Instruction::BinOp { op, lhs, rhs, .. } => {
                let lhs = self.interp_operand(lhs.clone())?;
                let rhs = self.interp_operand(rhs.clone())?;

                calculator::calculate_binary_operator_expression(&op, lhs, rhs).map_err(|_| {
                    InterpreterError::Misc {
                        func_name: self.stack_frame.func_name.clone(),
                        pc: self.stack_frame.pc,
                        msg: "calculate_binary_operator_expression".into(),
                    }
                })?
            }
            Instruction::UnaryOp { op, operand, .. } => {
                let operand = self.interp_operand(operand.clone())?;

                calculator::calculate_unary_operator_expression(&op, operand).map_err(|_| {
                    InterpreterError::Misc {
                        func_name: self.stack_frame.func_name.clone(),
                        pc: self.stack_frame.pc,
                        msg: "calculate_unary_operator_expression".into(),
                    }
                })?
            }
            Instruction::Store { ptr, value, .. } => {
                let ptr = self.interp_operand(ptr.clone())?;
                let value = self.interp_operand(value.clone())?;
                let (bid, offset, _) = self.interp_ptr(&ptr)?;
                self.memory
                    .store(bid, offset, &value, &self.ir.structs)
                    .map_err(|_| InterpreterError::Misc {
                        func_name: self.stack_frame.func_name.clone(),
                        pc: self.stack_frame.pc,
                        msg: format!(
                            "fail to store {:?} into memory with bid: {}, offset: {}",
                            value, bid, offset,
                        ),
                    })?;
                Value::Unit
            }
            Instruction::Load { ptr, .. } => {
                let ptr = self.interp_operand(ptr.clone())?;
                let (bid, offset, dtype) = self.interp_ptr(&ptr)?;
                self.memory.load(bid, offset, &dtype, &self.ir.structs)?
            }
            Instruction::Call { callee, args, .. } => {
                let ptr = self.interp_operand(callee.clone())?;

                // Get function name from pointer
                let (bid, _, _) = ptr.get_pointer().expect("`ptr` must be `Value::Pointer`");
                let bid = bid.expect("pointer for global variable must have bid value");
                let callee_name = self
                    .global_map
                    .get_var(bid)
                    .expect("bid must have relation with global variable");

                let func = self
                    .ir
                    .decls
                    .get(&callee_name)
                    .expect("function must be declared before being called");
                let (func_signature, func_def) = func
                    .get_function()
                    .expect("`func` must be function declaration");
                let func_def =
                    func_def
                        .as_ref()
                        .ok_or_else(|| InterpreterError::NoFunctionDefinition {
                            func_name: callee_name.clone(),
                        })?;

                let args = self.interp_args(func_signature, args)?;

                let stack_frame = StackFrame::new(func_def.bid_init, callee_name, func_def);
                let prev_stack_frame = mem::replace(&mut self.stack_frame, stack_frame);
                self.stack.push(prev_stack_frame);

                // Initialize state with function obtained by callee and args
                self.write_args(func_def.bid_init, args)?;
                self.alloc_local_variables()?;

                return Ok(());
            }
            Instruction::TypeCast {
                value,
                target_dtype,
            } => {
                let value = self.interp_operand(value.clone())?;
                calculator::calculate_typecast(value, target_dtype.clone()).map_err(|_| {
                    InterpreterError::Misc {
                        func_name: self.stack_frame.func_name.clone(),
                        pc: self.stack_frame.pc,
                        msg: "calculate_typecast".into(),
                    }
                })?
            }
            Instruction::GetElementPtr { ptr, offset, dtype } => {
                let ptr = self.interp_operand(ptr.clone())?;

                let (value, _, _) = self
                    .interp_operand(offset.clone())?
                    .get_int()
                    .expect("`idx` must be `Value::Int`");

                let (bid, prev_offset, ..) = ptr
                    .get_pointer()
                    .expect("`pointer` must be `Value::Pointer` to access memory");

                let inner_dtype = dtype
                    .get_pointer_inner()
                    .expect("`dtype` must be pointer type");

                let offset = prev_offset + value as isize;
                assert!(0 <= offset);

                Value::pointer(*bid, offset as isize, inner_dtype.clone())
            }
        };

        let register = RegisterId::temp(self.stack_frame.pc.bid, self.stack_frame.pc.iid);
        self.stack_frame.registers.write(register, result);
        self.stack_frame.pc.increment();

        Ok(())
    }

    fn interp_operand(&self, operand: Operand) -> Result<Value, InterpreterError> {
        match &operand {
            Operand::Constant(value) => Ok(self.interp_constant(value.clone())),
            Operand::Register { rid, .. } => {
                Ok(self.stack_frame.registers.read(rid.clone()).clone())
            }
        }
    }

    fn interp_constant(&self, value: Constant) -> Value {
        match value {
            Constant::GlobalVariable { name, dtype } => {
                let bid = self
                    .global_map
                    .get_bid(&name)
                    .expect("The name matching `bid` must exist.");

                // Generate appropriate pointer from `bid`
                Value::Pointer {
                    bid: Some(bid),
                    offset: 0,
                    dtype,
                }
            }
            constant => Value::try_from(constant).expect("constant must be transformed to value"),
        }
    }

    fn interp_ptr(&mut self, pointer: &Value) -> Result<(usize, isize, Dtype), InterpreterError> {
        let (bid, offset, dtype) = pointer
            .get_pointer()
            .ok_or_else(|| InterpreterError::Misc {
                func_name: self.stack_frame.func_name.clone(),
                pc: self.stack_frame.pc,
                msg: "Accessing memory with non-pointer".into(),
            })?;

        let bid = bid.ok_or_else(|| InterpreterError::Misc {
            func_name: self.stack_frame.func_name.clone(),
            pc: self.stack_frame.pc,
            msg: "Accessing memory with constant pointer".into(),
        })?;

        Ok((bid, *offset, dtype.clone()))
    }
}

#[inline]
pub fn interp(ir: &TranslationUnit, args: Vec<Value>) -> Result<Value, InterpreterError> {
    let mut init_state = State::new(ir, args)?;
    init_state.run()
}
