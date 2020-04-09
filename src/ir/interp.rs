use core::fmt;
use core::iter;
use core::mem;
use failure::Fail;
use std::collections::HashMap;

use itertools::izip;

use crate::ir::*;
use crate::*;

// TODO: the variants of Value will be added in the future
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
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
        value: f64,
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
    pub fn int(value: u128, width: usize, is_signed: bool) -> Self {
        Self::Int {
            value,
            width,
            is_signed,
        }
    }

    #[inline]
    fn float(value: f64, width: usize) -> Self {
        Self::Float { value, width }
    }

    #[inline]
    fn pointer(bid: Option<usize>, offset: isize, dtype: Dtype) -> Self {
        Self::Pointer { bid, offset, dtype }
    }

    #[inline]
    fn get_int(self) -> Option<(u128, usize, bool)> {
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
    fn default_from_dtype(dtype: &Dtype) -> Self {
        match dtype {
            ir::Dtype::Unit { .. } => Self::unit(),
            ir::Dtype::Int {
                width, is_signed, ..
            } => Self::int(u128::default(), *width, *is_signed),
            ir::Dtype::Float { width, .. } => Self::float(f64::default(), *width),
            ir::Dtype::Pointer { inner, .. } => Self::nullptr(inner.deref().clone()),
            ir::Dtype::Array { .. } => panic!("array type does not have a default value"),
            ir::Dtype::Function { .. } => panic!("function type does not have a default value"),
            ir::Dtype::Typedef { .. } => panic!("typedef should be replaced by real dtype"),
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
    use lang_c::ast;

    // TODO: change to template function in the future
    pub fn calculate_binary_operator_expression(
        op: &ast::BinaryOperator,
        lhs: Value,
        rhs: Value,
    ) -> Result<Value, ()> {
        match (op, lhs, rhs) {
            (_, Value::Undef { .. }, _) => Err(()),
            (_, _, Value::Undef { .. }) => Err(()),
            (
                op,
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

                match op {
                    // TODO: consider signed value in the future
                    ast::BinaryOperator::Plus => Ok(Value::int(lhs + rhs, lhs_w, lhs_s)),
                    ast::BinaryOperator::Minus => Ok(Value::int(lhs - rhs, lhs_w, lhs_s)),
                    ast::BinaryOperator::Multiply => Ok(Value::int(lhs * rhs, lhs_w, lhs_s)),
                    ast::BinaryOperator::Modulo => Ok(Value::int(lhs % rhs, lhs_w, lhs_s)),
                    ast::BinaryOperator::Equals => {
                        let result = if lhs == rhs { 1 } else { 0 };
                        Ok(Value::int(result, 1, false))
                    }
                    ast::BinaryOperator::NotEquals => {
                        let result = if lhs != rhs { 1 } else { 0 };
                        Ok(Value::int(result, 1, false))
                    }
                    ast::BinaryOperator::Less => {
                        // TODO: consider signed option
                        let result = if lhs < rhs { 1 } else { 0 };
                        Ok(Value::int(result, 1, false))
                    }
                    ast::BinaryOperator::Greater => {
                        // TODO: consider signed option
                        let result = if lhs > rhs { 1 } else { 0 };
                        Ok(Value::int(result, 1, false))
                    }
                    ast::BinaryOperator::LessOrEqual => {
                        // TODO: consider signed option
                        let result = if lhs <= rhs { 1 } else { 0 };
                        Ok(Value::int(result, 1, false))
                    }
                    ast::BinaryOperator::GreaterOrEqual => {
                        // TODO: consider signed option
                        let result = if lhs >= rhs { 1 } else { 0 };
                        Ok(Value::int(result, 1, false))
                    }
                    ast::BinaryOperator::LogicalAnd => {
                        assert!(lhs < 2);
                        assert!(rhs < 2);
                        let result = lhs | rhs;
                        Ok(Value::int(result, 1, lhs_s))
                    }
                    _ => todo!(
                        "calculate_binary_operator_expression: not supported operator {:?}",
                        op
                    ),
                }
            }
            _ => todo!(),
        }
    }

    pub fn calculate_unary_operator_expression(
        op: &ast::UnaryOperator,
        operand: Value,
    ) -> Result<Value, ()> {
        match (op, operand) {
            (_, Value::Undef { .. }) => Err(()),
            (
                ast::UnaryOperator::Plus,
                Value::Int {
                    value,
                    width,
                    is_signed,
                },
            ) => Ok(Value::int(value, width, is_signed)),
            (
                ast::UnaryOperator::Minus,
                Value::Int {
                    value,
                    width,
                    is_signed,
                },
            ) => {
                assert!(is_signed);
                let result = -(value as i128);
                Ok(Value::int(result as u128, width, is_signed))
            }
            (
                ast::UnaryOperator::Negate,
                Value::Int {
                    value,
                    width,
                    is_signed,
                },
            ) => {
                // Check if it is boolean
                assert!(width == 1);
                let result = if value == 0 { 1 } else { 0 };
                Ok(Value::int(result, width, is_signed))
            }
            _ => todo!(),
        }
    }

    pub fn calculate_typecast(value: Value, dtype: crate::ir::Dtype) -> Result<Value, ()> {
        match (value, dtype) {
            (Value::Undef { .. }, _) => Err(()),
            // TODO: distinguish zero/signed extension in the future
            // TODO: consider truncate in the future
            (
                Value::Int { value, .. },
                crate::ir::Dtype::Int {
                    width, is_signed, ..
                },
            ) => Ok(Value::int(value, width, is_signed)),
            (Value::Float { value, .. }, crate::ir::Dtype::Float { width, .. }) => {
                Ok(Value::float(value, width))
            }
            (value, dtype) => todo!("calculate_typecast ({:?}) {:?}", dtype, value),
        }
    }
}

// TODO
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum Byte {
    Undef,
    Concrete(u8),
    Pointer {
        bid: usize,
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
    fn pointer(bid: usize, offset: isize, index: usize) -> Self {
        Self::Pointer { bid, offset, index }
    }

    fn get_concrete(&self) -> Option<u8> {
        if let Self::Concrete(byte) = self {
            Some(*byte)
        } else {
            None
        }
    }

    fn get_pointer(&self) -> Option<(usize, isize, usize)> {
        if let Self::Pointer { bid, offset, index } = self {
            Some((*bid, *offset, *index))
        } else {
            None
        }
    }

    fn block_from_dtype(dtype: &Dtype) -> Vec<Self> {
        let size = dtype.size_align_of().unwrap().0;
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

    fn bytes_to_value<'b, I>(bytes: &mut I, dtype: &Dtype) -> Result<Value, InterpreterError>
    where
        I: Iterator<Item = &'b Self>,
    {
        match dtype {
            ir::Dtype::Unit { .. } => Ok(Value::Unit),
            ir::Dtype::Int {
                width, is_signed, ..
            } => {
                let value = some_or!(
                    bytes
                        .by_ref()
                        .take(*width)
                        .map(|b| b.get_concrete())
                        .collect::<Option<Vec<_>>>(),
                    return Ok(Value::undef(dtype.clone()))
                );
                let value = Self::bytes_to_u128(&value, *is_signed);
                Ok(Value::int(value, *width, *is_signed))
            }
            ir::Dtype::Float { width, .. } => {
                let value = some_or!(
                    bytes
                        .by_ref()
                        .take(*width)
                        .map(|b| b.get_concrete())
                        .collect::<Option<Vec<_>>>(),
                    return Ok(Value::undef(dtype.clone()))
                );
                let value = Self::bytes_to_u128(&value, false);
                let value = if *width == Dtype::SIZE_OF_FLOAT {
                    f32::from_bits(value as u32) as f64
                } else {
                    f64::from_bits(value as u64)
                };

                Ok(Value::float(value, *width))
            }
            ir::Dtype::Pointer { inner, .. } => {
                let value = some_or!(
                    bytes
                        .by_ref()
                        .take(Dtype::SIZE_OF_POINTER)
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
                        Value::pointer(Some(*bid), *offset, inner.deref().clone())
                    },
                )
            }
            ir::Dtype::Array { inner, size } => {
                let (inner_size, inner_align) = inner.size_align_of().unwrap();
                let padding = std::cmp::max(inner_size, inner_align) - inner_size;
                let values = (0..*size)
                    .map(|_| {
                        let value = Self::bytes_to_value(bytes, inner)?;
                        let _ = bytes.by_ref().take(padding);
                        Ok(value)
                    })
                    .collect::<Result<Vec<_>, InterpreterError>>()?;
                Ok(Value::Array {
                    inner_dtype: inner.deref().clone(),
                    values,
                })
            }
            ir::Dtype::Function { .. } => panic!("function value cannot be constructed from bytes"),
            ir::Dtype::Typedef { .. } => panic!("typedef should be replaced by real dtype"),
        }
    }

    fn value_to_bytes(value: &Value) -> Vec<Self> {
        match value {
            Value::Undef { dtype } => Self::block_from_dtype(dtype),
            Value::Unit => Vec::new(),
            Value::Int { value, width, .. } => {
                let size = (*width + Dtype::BITS_OF_BYTE - 1) / Dtype::BITS_OF_BYTE;
                Self::u128_to_bytes(*value, size)
                    .iter()
                    .map(|b| Self::concrete(*b))
                    .collect::<Vec<_>>()
            }
            Value::Float { value, width } => {
                let size = (*width + Dtype::BITS_OF_BYTE - 1) / Dtype::BITS_OF_BYTE;
                let value: u128 = match size {
                    Dtype::SIZE_OF_FLOAT => (*value as f32).to_bits() as u128,
                    Dtype::SIZE_OF_DOUBLE => (*value as f64).to_bits() as u128,
                    _ => panic!("value_to_bytes: {} is not a valid float size", size),
                };

                Self::u128_to_bytes(value, size)
                    .iter()
                    .map(|b| Self::concrete(*b))
                    .collect::<Vec<_>>()
            }
            Value::Pointer { bid, offset, .. } => (0..Dtype::SIZE_OF_POINTER)
                .map(|i| Self::pointer(bid.unwrap(), *offset, i))
                .collect(),
            Value::Array {
                inner_dtype,
                values,
            } => {
                let (inner_size, inner_align) = inner_dtype.size_align_of().unwrap();
                let padding = std::cmp::max(inner_size, inner_align) - inner_size;
                values
                    .iter()
                    .map(|v| {
                        let mut result = Self::value_to_bytes(v);
                        result.extend(iter::repeat(Byte::Undef).take(padding));
                        result
                    })
                    .flatten()
                    .collect()
            }
        }
    }
}

impl Memory {
    fn alloc(&mut self, dtype: &Dtype) -> Result<usize, InterpreterError> {
        let bid = self.inner.len();
        self.inner.push(Some(Byte::block_from_dtype(dtype)));
        Ok(bid)
    }

    fn dealloc(
        &mut self,
        bid: usize,
        offset: isize,
        dtype: &Dtype,
    ) -> Result<(), InterpreterError> {
        let block = &mut self.inner[bid];
        assert_eq!(offset, 0);
        assert_eq!(
            block.as_mut().unwrap().len(),
            dtype.size_align_of().unwrap().0
        );
        *block = None;
        Ok(())
    }

    fn load(&self, bid: usize, offset: isize, dtype: &Dtype) -> Result<Value, InterpreterError> {
        assert!(0 <= offset);
        let offset = offset as usize;
        let size = dtype.size_align_of().unwrap().0;
        let mut iter = self.inner[bid].as_ref().unwrap()[offset..(offset + size)].iter();
        Byte::bytes_to_value(&mut iter, dtype)
    }

    fn store(&mut self, bid: usize, offset: isize, value: &Value) {
        assert!(0 <= offset);
        let offset = offset as usize;
        let size = value.dtype().size_align_of().unwrap().0;
        let bytes = Byte::value_to_bytes(value);
        self.inner[bid]
            .as_mut()
            .unwrap()
            .splice(offset..(offset + size), bytes.iter().cloned());
    }
}

// TODO: allocation fields will be added in the future
// TODO: program fields will be added in the future
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
            let bid = self.memory.alloc(&decl.dtype())?;
            self.global_map.insert(name.clone(), bid)?;

            // Initialize allocated memory space
            match decl {
                Declaration::Variable { dtype, initializer } => match &dtype {
                    ir::Dtype::Unit { .. } => (),
                    ir::Dtype::Int { .. } | ir::Dtype::Float { .. } | ir::Dtype::Pointer { .. } => {
                        let value = if let Some(constant) = initializer {
                            self.interp_constant(constant.clone())
                        } else {
                            Value::default_from_dtype(&dtype)
                        };

                        self.memory.store(bid, 0, &value);
                    }
                    ir::Dtype::Array { .. } => todo!("Initializer::List is needed"),
                    ir::Dtype::Function { .. } => panic!("function variable does not exist"),
                    ir::Dtype::Typedef { .. } => panic!("typedef should be replaced by real dtype"),
                },
                // If functin declaration, skip initialization
                Declaration::Function { .. } => (),
            }
        }

        Ok(())
    }

    fn alloc_local_variables(&mut self) -> Result<(), InterpreterError> {
        // add alloc register
        for (id, allocation) in self.stack_frame.func_def.allocations.iter().enumerate() {
            let bid = self.memory.alloc(&allocation)?;
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
            self.memory.dealloc(bid.unwrap(), *offset, dtype)?;
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

        for (i, a) in arg.args.iter().enumerate() {
            let v = self.interp_operand(a.clone()).unwrap();
            self.stack_frame
                .registers
                .write(RegisterId::arg(arg.bid, i), v);
        }

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
                self.memory.store(bid, offset, &value);
                Value::Unit
            }
            Instruction::Load { ptr, .. } => {
                let ptr = self.interp_operand(ptr.clone())?;
                let (bid, offset, dtype) = self.interp_ptr(&ptr)?;
                self.memory.load(bid, offset, &dtype)?
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
            Constant::Undef { dtype } => Value::Undef { dtype },
            Constant::Unit => Value::Unit,
            Constant::Int {
                value,
                width,
                is_signed,
            } => Value::Int {
                value,
                width,
                is_signed,
            },
            Constant::Float { value, width } => Value::Float {
                value: value.into_inner(),
                width,
            },
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
