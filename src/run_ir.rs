use crate::ir::*;
use crate::*;

use failure::Fail;
use std::collections::HashMap;
use std::mem;

use itertools::izip;

// TODO: the variants of Value will be added in the future
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Undef,
    Unit,
    Int(i32),
    Float(f32),
    Bool(bool),
    Pointer { bid: Option<usize>, offset: usize },
}

impl Value {
    #[inline]
    fn pointer(bid: Option<usize>, offset: usize) -> Self {
        Self::Pointer { bid, offset }
    }

    #[inline]
    fn get_bool(self) -> Option<bool> {
        if let Value::Bool(value) = self {
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    fn get_pointer(self) -> Option<(Option<usize>, usize)> {
        if let Value::Pointer { bid, offset } = self {
            Some((bid, offset))
        } else {
            None
        }
    }

    #[inline]
    fn nullptr() -> Self {
        Self::Pointer {
            bid: None,
            offset: 0,
        }
    }

    #[inline]
    fn default_from_dtype(dtype: &Dtype) -> Self {
        match dtype {
            // TODO: consider `Unit` value in the future
            ir::Dtype::Unit { .. } => todo!(),
            ir::Dtype::Int { width, .. } => match width {
                32 => Self::Int(i32::default()),
                _ => todo!("other cases will be covered"),
            },
            ir::Dtype::Float { .. } => Self::Float(f32::default()),
            ir::Dtype::Pointer { .. } => Self::nullptr(),
            ir::Dtype::Function { .. } => panic!("function types do not have a default value"),
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
        display = "{}:{}:{} / Undef value cannot be used as an operand",
        func_name, bid, iid
    )]
    Misc {
        func_name: String,
        bid: BlockId,
        iid: usize,
    },
}

#[derive(Debug, PartialEq, Clone)]
struct Pc {
    pub bid: BlockId,
    pub iid: usize,
}

impl Pc {
    fn new(bid: BlockId) -> Pc {
        Pc { bid, iid: 0 }
    }

    fn increment(&mut self) {
        self.iid += 1;
    }
}

#[derive(Debug, PartialEq, Clone)]
struct RegisterMap {
    inner: HashMap<RegisterId, Value>,
}

impl RegisterMap {
    fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
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
            registers: RegisterMap::new(),
            func_name,
            func_def,
        }
    }
}

mod calculator {
    use super::Value;
    use lang_c::ast;

    pub fn calculate_binary_operator_expression(
        op: &ast::BinaryOperator,
        lhs: Value,
        rhs: Value,
    ) -> Result<Value, ()> {
        match (op, lhs, rhs) {
            (_, Value::Undef, _) => Err(()),
            (_, _, Value::Undef) => Err(()),
            (ast::BinaryOperator::Plus, Value::Int(lhs), Value::Int(rhs)) => {
                Ok(Value::Int(lhs + rhs))
            }
            (ast::BinaryOperator::Minus, Value::Int(lhs), Value::Int(rhs)) => {
                Ok(Value::Int(lhs - rhs))
            }
            (ast::BinaryOperator::Equals, Value::Int(lhs), Value::Int(rhs)) => {
                Ok(Value::Bool(lhs == rhs))
            }
            (ast::BinaryOperator::NotEquals, Value::Int(lhs), Value::Int(rhs)) => {
                Ok(Value::Bool(lhs != rhs))
            }
            (ast::BinaryOperator::Less, Value::Int(lhs), Value::Int(rhs)) => {
                Ok(Value::Bool(lhs < rhs))
            }
            (ast::BinaryOperator::GreaterOrEqual, Value::Int(lhs), Value::Int(rhs)) => {
                Ok(Value::Bool(lhs >= rhs))
            }
            _ => todo!(),
        }
    }

    pub fn calculate_unary_operator_expression(
        op: &ast::UnaryOperator,
        operand: Value,
    ) -> Result<Value, ()> {
        match (op, operand) {
            (_, Value::Undef) => Err(()),
            (ast::UnaryOperator::Plus, Value::Int(value)) => Ok(Value::Int(value)),
            (ast::UnaryOperator::Minus, Value::Int(value)) => Ok(Value::Int(-value)),
            (ast::UnaryOperator::Negate, Value::Bool(value)) => Ok(Value::Bool(!value)),
            _ => todo!(),
        }
    }

    pub fn calculate_typecast(value: &Value, dtype: &crate::ir::Dtype) -> Result<Value, ()> {
        match (value, dtype) {
            (Value::Int(_), crate::ir::Dtype::Int { .. }) => Ok(value.clone()),
            (Value::Bool(b), crate::ir::Dtype::Int { .. }) => {
                Ok(Value::Int(if *b { 1 } else { 0 }))
            }
            _ => todo!("calculate_typecast ({:?}) {:?}", dtype, value),
        }
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
    // TODO: memory type should change to Vec<Vec<Byte>>
    pub memory: Vec<Vec<Value>>,
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
            memory: Vec::new(),
            ir,
        };

        state.alloc_global_variable()?;

        // Initialize state with main function and args
        state.pass_arguments(args)?;
        state.alloc_local_variable()?;

        Ok(state)
    }

    fn alloc_global_variable(&mut self) -> Result<(), InterpreterError> {
        for (name, decl) in &self.ir.decls {
            // Memory allocation
            let bid = self.alloc_memory(&decl.dtype())?;
            self.global_map.insert(name.clone(), bid)?;

            // Initialize allocated memory space
            match decl {
                Declaration::Variable { dtype, initializer } => {
                    let value = if let Some(constant) = initializer {
                        self.constant_to_value(constant.clone())
                    } else {
                        Value::default_from_dtype(dtype)
                    };

                    self.memory[bid][0] = value;
                }
                // If functin declaration, skip initialization
                Declaration::Function { .. } => (),
            }
        }

        Ok(())
    }

    fn pass_arguments(&mut self, args: Vec<Value>) -> Result<(), InterpreterError> {
        for (i, value) in args.iter().enumerate() {
            self.register_write(RegisterId::arg(i), value.clone());
        }

        Ok(())
    }

    fn alloc_local_variable(&mut self) -> Result<(), InterpreterError> {
        // add alloc register
        for (id, allocation) in self.stack_frame.func_def.allocations.iter().enumerate() {
            let bid = self.alloc_memory(&allocation)?;
            let ptr = Value::pointer(Some(bid), 0);
            let rid = RegisterId::local("".to_string(), id);

            self.register_write(rid, ptr)
        }

        Ok(())
    }

    fn alloc_memory(&mut self, dtype: &Dtype) -> Result<usize, InterpreterError> {
        // TODO: memory block will be handled as Vec<Byte>
        let memory_block = match dtype {
            Dtype::Unit { .. } => vec![],
            Dtype::Int { width, .. } => match width {
                32 => vec![Value::Undef],
                _ => todo!(),
            },
            Dtype::Float { .. } => todo!(),
            Dtype::Pointer { .. } => vec![Value::Undef],
            Dtype::Function { .. } => vec![],
        };

        self.memory.push(memory_block);

        Ok(self.memory.len() - 1)
    }

    fn preprocess_args(
        &self,
        signature: &FunctionSignature,
        args: &[Operand],
    ) -> Result<Vec<Value>, InterpreterError> {
        // Check that the dtype of each args matches the expected
        if !(args.len() == signature.params.len()
            && izip!(args, &signature.params).all(|(a, d)| a.dtype().is_compatible(d)))
        {
            panic!("dtype of args and params must be compatible")
        }

        args.iter()
            .map(|a| self.get_value(a.clone()))
            .collect::<Result<Vec<_>, _>>()
    }

    fn step(&mut self) -> Result<Option<Value>, InterpreterError> {
        let block = self
            .stack_frame
            .func_def
            .blocks
            .get(&self.stack_frame.pc.bid)
            .expect("block matched with `bid` must be exist");

        if block.instructions.len() == self.stack_frame.pc.iid {
            self.interpret_block_exit(&block.exit)
        } else {
            let instr = block
                .instructions
                .get(self.stack_frame.pc.iid)
                .expect("instruction matched with `iid` must be exist");

            self.interpret_instruction(instr)
        }
    }

    fn run(&mut self) -> Result<Value, InterpreterError> {
        loop {
            if let Some(value) = self.step()? {
                // TODO: Before return, free memory allocated in a function

                // restore previous state
                let prev_stack_frame = some_or!(self.stack.pop(), {
                    return Ok(value);
                });
                self.stack_frame = prev_stack_frame;

                // create temporary register to write return value
                let register = RegisterId::temp(self.stack_frame.pc.bid, self.stack_frame.pc.iid);
                self.register_write(register, value);
                self.stack_frame.pc.increment();
            }
        }
    }

    fn interpret_block_exit(
        &mut self,
        block_exit: &BlockExit,
    ) -> Result<Option<Value>, InterpreterError> {
        match block_exit {
            BlockExit::Jump { bid } => {
                self.stack_frame.pc = Pc::new(*bid);
                Ok(None)
            }
            BlockExit::ConditionalJump {
                condition,
                bid_then,
                bid_else,
            } => {
                let value = self.get_value(condition.clone())?;
                let value = value.get_bool().expect("`condition` must be `Value::Bool`");

                self.stack_frame.pc = Pc::new(if value { *bid_then } else { *bid_else });
                Ok(None)
            }
            BlockExit::Switch {
                value,
                default,
                cases,
            } => {
                let value = self.get_value(value.clone())?;

                // TODO: consider different integer `width` in the future
                let bid_next = cases
                    .iter()
                    .find(|(c, _)| value == self.constant_to_value(c.clone()))
                    .map(|(_, bid)| bid)
                    .unwrap_or_else(|| default);

                self.stack_frame.pc = Pc::new(*bid_next);

                Ok(None)
            }
            BlockExit::Return { value } => Ok(Some(self.get_value(value.clone())?)),
            BlockExit::Unreachable => Err(InterpreterError::Unreachable),
        }
    }

    fn interpret_instruction(
        &mut self,
        instruction: &Instruction,
    ) -> Result<Option<Value>, InterpreterError> {
        let result = match instruction {
            Instruction::BinOp { op, lhs, rhs, .. } => {
                let lhs = self.get_value(lhs.clone())?;
                let rhs = self.get_value(rhs.clone())?;

                calculator::calculate_binary_operator_expression(&op, lhs, rhs).map_err(|_| {
                    InterpreterError::Misc {
                        func_name: self.stack_frame.func_name.clone(),
                        bid: self.stack_frame.pc.bid,
                        iid: self.stack_frame.pc.iid,
                    }
                })?
            }
            Instruction::UnaryOp { op, operand, .. } => {
                let operand = self.get_value(operand.clone())?;

                calculator::calculate_unary_operator_expression(&op, operand).map_err(|_| {
                    InterpreterError::Misc {
                        func_name: self.stack_frame.func_name.clone(),
                        bid: self.stack_frame.pc.bid,
                        iid: self.stack_frame.pc.iid,
                    }
                })?
            }
            Instruction::Store { ptr, value, .. } => {
                let ptr = self.get_value(ptr.clone())?;
                let value = self.get_value(value.clone())?;

                self.memory_store(ptr, value)?;

                Value::Unit
            }
            Instruction::Load { ptr, .. } => {
                let ptr = self.get_value(ptr.clone())?;

                self.memory_load(ptr)?
            }
            Instruction::Call { callee, args, .. } => {
                let ptr = self.get_value(callee.clone())?;

                // Get function name from pointer
                let (bid, _) = ptr.get_pointer().expect("`ptr` must be `Value::Pointer`");
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

                let args = self.preprocess_args(func_signature, args)?;

                let stack_frame = StackFrame::new(func_def.bid_init, callee_name, func_def);
                let prev_stack_frame = mem::replace(&mut self.stack_frame, stack_frame);
                self.stack.push(prev_stack_frame);

                // Initialize state with function obtained by callee and args
                self.pass_arguments(args)?;
                self.alloc_local_variable()?;

                return Ok(None);
            }
            Instruction::TypeCast {
                value,
                target_dtype,
            } => {
                let value = self.get_value(value.clone())?;
                calculator::calculate_typecast(&value, target_dtype).map_err(|_| {
                    InterpreterError::Misc {
                        func_name: self.stack_frame.func_name.clone(),
                        bid: self.stack_frame.pc.bid,
                        iid: self.stack_frame.pc.iid,
                    }
                })?
            }
        };

        let register = RegisterId::temp(self.stack_frame.pc.bid, self.stack_frame.pc.iid);
        self.register_write(register, result);
        self.stack_frame.pc.increment();

        Ok(None)
    }

    fn get_value(&self, operand: Operand) -> Result<Value, InterpreterError> {
        match &operand {
            Operand::Constant(value) => Ok(self.constant_to_value(value.clone())),
            Operand::Register { rid, .. } => Ok(self.register_read(rid.clone())),
        }
    }

    fn constant_to_value(&self, value: Constant) -> Value {
        match value {
            Constant::Unit => Value::Unit,
            // TODO: consider `width` and `is_signed` in the future
            Constant::Int { value, .. } => Value::Int(value as i32),
            Constant::Float { value, .. } => Value::Float(value as f32),
            Constant::GlobalVariable { name, .. } => {
                let bid = self
                    .global_map
                    .get_bid(&name)
                    .expect("The name matching `bid` must exist.");

                // Generate appropriate pointer from `bid`
                Value::Pointer {
                    bid: Some(bid),
                    offset: 0,
                }
            }
        }
    }

    fn register_write(&mut self, rid: RegisterId, value: Value) {
        let _ = self.stack_frame.registers.inner.insert(rid, value);
    }

    fn register_read(&self, rid: RegisterId) -> Value {
        self.stack_frame
            .registers
            .inner
            .get(&rid)
            .cloned()
            .expect("`rid` must be assigned before it can be used")
    }

    fn memory_store(&mut self, pointer: Value, value: Value) -> Result<(), InterpreterError> {
        let (bid, offset) = pointer
            .get_pointer()
            .expect("`pointer` must be `Value::Pointer` to access memory");

        let bid = bid.expect("write to memory using constant value address is not allowed");
        self.memory[bid][offset] = value;

        Ok(())
    }

    fn memory_load(&self, pointer: Value) -> Result<Value, InterpreterError> {
        let (bid, offset) = pointer
            .get_pointer()
            .expect("`pointer` must be `Value::Pointer` to access memory");

        let bid = bid.expect("read from memory using constant value address is not allowed");

        Ok(self.memory[bid][offset].clone())
    }
}

#[inline]
pub fn run_ir(ir: &TranslationUnit, args: Vec<Value>) -> Result<Value, InterpreterError> {
    let mut init_state = State::new(ir, args)?;
    init_state.run()
}
