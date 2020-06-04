use std::fs;
use std::path::Path;

use lang_c::ast::{BinaryOperator, UnaryOperator};

use crate::ir::*;
use crate::Translate;

peg::parser! {
    grammar ir_parse() for str {
        rule whitespace() = quiet!{[' ' | '\n' | '\t']}

        rule _() = whitespace()*

        rule __() = whitespace()+

        pub rule translation_unit() -> TranslationUnit
            = _ ds:(named_decl() ** __) _ {
                let mut decls = BTreeMap::new();
                for decl in ds {
                    let result = decls.insert(decl.name.unwrap(), decl.inner);
                    assert!(result.is_none());
                }
                TranslationUnit { decls, structs: HashMap::new() }
            }

        rule named_decl() -> Named<Declaration> =
            "var" __ dtype:dtype() __ var:global_variable() _ "=" _ initializer:initializer() {
                Named::new(Some(var), Declaration::Variable {
                    dtype: dtype,
                    initializer,
                })
            }
        /
            "fun" __ dtype:dtype() __ var:global_variable() _ "(" params:(dtype() ** (_ "," _)) _ ")" _ "{" _ fun_body:fun_body() _ "}" {
                Named::new(Some(var), Declaration::Function {
                    signature: FunctionSignature::new(Dtype::function(dtype, params)),
                    definition: Some(fun_body),
                })
            }
        /
            "fun" __ dtype:dtype() __ var:global_variable() _ "(" params:(dtype() ** (_ "," _)) _ ")" {
                Named::new(Some(var), Declaration::Function {
                    signature: FunctionSignature::new(Dtype::function(dtype, params)),
                    definition: None,
                })
            }

        rule dtype() -> Dtype =
            "unit" { Dtype::unit() }
        /
            "u" n:number() { Dtype::int(n).set_signed(false) }
        /
            "i" n:number() { Dtype::int(n) }
        /
            "f" n:number() { Dtype::float(n) }
        /
            "*" _ "const" _ inner:dtype() { Dtype::pointer(inner).set_const(true) }
        /
            "*" _ inner:dtype() { Dtype::pointer(inner) }
        /
            "[" _ n:number() __ "x" __ inner:dtype() _ "]" {
                Dtype::Array { inner: Box::new(inner), size: n }
            }
        /
            "[ret:" _ ret:dtype() __ "params:(" params:(dtype() ** (_ "," _)) _ ")]" {
                Dtype::Function { ret: Box::new(ret), params }
            }
        /
            "const" __ dtype:dtype() { dtype.set_const(true) }
        /
            expected!("dtype")

        rule id() -> String
            = n:$(['_' | 'a'..='z' | 'A'..='Z']['_' | 'a'..='z' | 'A'..='Z' | '0'..='9']*) {
                String::from(n)
            }
        / expected!("id")

        rule global_variable() -> String
            = "@" id:id() {
                id
            }
        / expected!("global-variable")

        rule arg() -> usize // TODO
            = "<arg>" {
                todo!()
            }

        rule fun_body() -> FunctionDefinition
            = "init:" __ "bid:" _ bid_init:bid() _ "allocations:" _ allocations:(allocation() ** __) _ blocks:(block() ** __) {
                FunctionDefinition {
                    allocations: allocations.into_iter().map(|a| a.1).collect(),
                    blocks: blocks.into_iter().collect(),
                    bid_init,
                }
            }

        rule allocation() -> (usize, Named<Dtype>)
            = "%l" number:number() ":" dtype:dtype() ":" name:id() {
                (number, Named::new(Some(name), dtype))
            }

        rule block() -> (BlockId, Block)
            = "block" __ bid:bid() _ ":" _ phinodes:(phinode() ** __) _ instructions:(instruction() ** __) _ exit:exit() {
                if !phinodes.iter().enumerate().all(|(i1, (bid2, i2, _))| bid == *bid2 && i1 == *i2) {
                    panic!("Phinode id mismatches");
                }

                if !instructions.iter().enumerate().all(|(i1, (bid2, i2, _))| bid == *bid2 && i1 == *i2) {
                    panic!("Instruction id mismatches");
                }

                (bid,
                 Block {
                     phinodes: phinodes.into_iter().map(|(_, _, phi)| phi).collect(),
                     instructions: instructions.into_iter().map(|(_, _, instr)| instr).collect(),
                     exit,
                 })
            }

        rule number() -> usize
            = n:$(['0'..='9']+) {
                n.parse().unwrap()
            }
        / expected!("number")

        rule float_number() -> f64
            = f:$(['0'..='9']+['.']['0'..='9']+) {
                f.parse().unwrap()
            }
        / expected!("float_number")

        rule bid() -> BlockId
            = "b" n:number() {
                BlockId(n)
            }
        / expected!("bid")

        rule phinode() -> (BlockId, usize, Named<Dtype>)
            = "%" bid:bid() ":p" number:number() ":" dtype:dtype() name:(":" name:id() { name })? {
                (bid, number, Named::new(name, dtype))
            }
        / expected!("phinode")

        rule instruction() -> (BlockId, usize, Named<Instruction>)
            = "%" bid:bid() ":i" number:number() ":" dtype:dtype() name:(":" name:id() { name })? _ "=" _ instruction:instruction_inner() {
                // TODO: The dtype of `GetElementPtr` instruction depends on the situation.
                // Let's `ptr` has `*[5 x i32]` type, after applying `GetElementPtr` instruction,
                // the dtype of the result can be `*i32` or `*[5 x i32]` in the current KECC.
                // For this reason, we need to check the dtype of the result to confirm the dtype
                // of `GetElementPtr` instruction when parsing IR.
                let instruction = if let Instruction::GetElementPtr { ptr, offset, .. } = instruction {
                    Instruction::GetElementPtr { ptr, offset, dtype: Box::new(dtype) }
                } else {
                    instruction
                };

                (bid, number, Named::new(name, instruction))
            }
        / expected!("instruction")

        rule instruction_inner() -> Instruction =
            "nop" {
                Instruction::Nop
            }
        /
            "load" __ ptr:operand() {
                Instruction::Load { ptr }
            }
        /
            "store" __ value:operand() __ ptr:operand() {
                Instruction::Store { ptr, value }
            }
        /
            "call" __ callee:operand() _ "(" _ args:(operand() ** (_ "," _)) _ ")" {
                let dtype_of_callee = callee.dtype();
                let function_type = dtype_of_callee
                    .get_pointer_inner()
                    .expect("`callee`'s dtype must be function pointer type");
                let return_type = function_type
                    .get_function_inner()
                    .expect("`callee`'s dtype must be function pointer type")
                    .0.clone();

                Instruction::Call {
                    callee,
                    args,
                    return_type,
                }
            }
        /
            "typecast" __ value:operand() __ "to" __ target_dtype:dtype() {
                Instruction::TypeCast { value, target_dtype }
            }
        /
            op:unary_op() __ operand:operand() {
                let dtype = operand.dtype();
                Instruction::UnaryOp {
                    op,
                    operand,
                    dtype,
                }
            }
        /
            op:arith_op() __ lhs:operand() __ rhs:operand() {
                let dtype = lhs.dtype();
                assert_eq!(&dtype, &rhs.dtype());
                Instruction::BinOp {
                    op,
                    lhs,
                    rhs,
                    dtype,
                }
            }
        /
            op:shift_op() __ lhs:operand() __ rhs:operand() {
                let dtype = lhs.dtype();
                assert_eq!(&dtype, &rhs.dtype());
                Instruction::BinOp {
                    op,
                    lhs,
                    rhs,
                    dtype,
                }
            }
        /
            "cmp" __ op:comparison_op() __ lhs:operand() __ rhs:operand() {
                assert_eq!(lhs.dtype(), rhs.dtype());
                Instruction::BinOp {
                    op,
                    lhs,
                    rhs,
                    dtype: Dtype::BOOL,
                }
            }
        /
            op:bitwise_op() __ lhs:operand() __ rhs:operand() {
                let dtype = lhs.dtype();
                assert_eq!(&dtype, &rhs.dtype());
                Instruction::BinOp {
                    op,
                    lhs,
                    rhs,
                    dtype,
                }
            }
        /
            "getelementptr" __ ptr:operand() __ "offset" __ offset:operand() {
                Instruction::GetElementPtr{
                    ptr,
                    offset,
                    dtype: Box::new(Dtype::unit()), // TODO
                }
            }
        /
            "<instruction>" {
                todo!()
            }
        / expected!("instruction_inner")

        rule arith_op() -> BinaryOperator =
            "add" { BinaryOperator::Plus }
        /
            "sub" { BinaryOperator::Minus }
        /
            "mul" { BinaryOperator::Multiply }
        /
            "div" { BinaryOperator::Divide }
        /
            "mod" { BinaryOperator::Modulo }

        rule shift_op() -> BinaryOperator =
            "shl" { BinaryOperator::ShiftLeft }
        /
            "shr" { BinaryOperator::ShiftRight }

        rule comparison_op() -> BinaryOperator =
            "eq" { BinaryOperator::Equals }
        /
            "ne" { BinaryOperator::NotEquals }
        /
            "lt" { BinaryOperator::Less }
        /
            "le" { BinaryOperator::LessOrEqual }
        /
            "gt" { BinaryOperator::Greater }
        /
            "ge" { BinaryOperator::GreaterOrEqual }

        rule bitwise_op() -> BinaryOperator =
            "and" { BinaryOperator::BitwiseAnd }
        /
            "xor" { BinaryOperator::BitwiseXor }
        /
            "or" { BinaryOperator::BitwiseOr }

        rule unary_op() -> UnaryOperator =
            "plus" { UnaryOperator::Plus }
        /
            "minus" { UnaryOperator::Minus }
        /
            "negate" { UnaryOperator::Negate }

        rule exit() -> BlockExit =
            "j" __ arg:jump_arg() {
                BlockExit::Jump { arg }
            }
        /
            "br" __ condition:operand() _ "," _ arg_then:jump_arg() _ "," _ arg_else:jump_arg() {
                BlockExit::ConditionalJump { condition, arg_then: Box::new(arg_then), arg_else: Box::new(arg_else) }
            }
        /
            "switch" __ value:operand() __ "default" __ default:jump_arg() _ "[" _ cases:(switch_case() ** __) _ "]" {
                BlockExit::Switch { value, default: Box::new(default), cases }
            }
        /
            "ret" __ value:operand() {
                BlockExit::Return { value }
            }
        /
            "unreachable" {
                BlockExit::Unreachable
            }

        rule constant() -> Constant =
            f:float_number() {
                Constant::float(f, Dtype::float(64)) // TODO: the right dtype
            }
        /
            n:number() {
                Constant::int(n as _, Dtype::int(128)) // TODO: the right dtype
            }
        /
            "undef" {
                Constant::undef(Dtype::unit()) // TODO
            }
        /
            "unit" {
                Constant::unit()
            }
        /
            name:global_variable() {
                Constant::GlobalVariable {
                    name,
                    dtype: Dtype::unit(), // TODO
                }
            }
        /
            "<constant>" {
                todo!()
            }


        rule register_id() -> RegisterId =
            "%l" id:number() {
                RegisterId::local(id)
            }
        /
            "%" bid:bid() ":p" id:number() {
                RegisterId::arg(bid, id)
            }
        /
            "%" bid:bid() ":i" id:number() {
                RegisterId::temp(bid, id)
            }

        rule operand() -> Operand =
            constant:constant() ":" dtype:dtype() {
                let constant = match (&constant, &dtype) {
                    (Constant::Int { value, .. }, Dtype::Int { width, is_signed, .. }) => {
                        Constant::Int {
                            value: *value,
                            width: *width,
                            is_signed: *is_signed,
                        }
                    }
                    (Constant::Float { value, .. }, Dtype::Float { width, .. }) => {
                        Constant::Float {
                            value: *value,
                            width: *width,
                        }
                    }
                    (Constant::Undef { .. }, _) => {
                        Constant::undef(dtype.clone())
                    }
                    (Constant::GlobalVariable { name, .. }, _) => {
                        let dtype_of_inner = dtype.get_pointer_inner().expect("`dtype` must be pointer type");
                        Constant::global_variable(name.clone(), dtype_of_inner.clone())
                    }
                    _ => constant.clone(),
                };
                Operand::Constant(constant)
            }
        /
            rid:register_id() ":" dtype:dtype() {
                Operand::Register { rid, dtype }
            }

        rule jump_arg() -> JumpArg
            = bid:bid() _ "(" _ args:(operand() ** (_ "," _)) _ ")" {
                JumpArg { bid, args }
            }

        rule switch_case() -> (Constant, JumpArg)
            = operand:operand() __ jump_arg:jump_arg() {
                let constant = operand.get_constant().unwrap().clone();
                (constant, jump_arg)
            }

        rule initializer() -> Option<lang_c::ast::Initializer> =
            "default" {
                None
            }
        /
            "<initializer>" {
                todo!()
            }
    }
}

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    ParseError(peg::error::ParseError<peg::str::LineCol>),
    ResolveError,
}

#[derive(Default)]
pub struct Parse {}

impl<P: AsRef<Path>> Translate<P> for Parse {
    type Target = TranslationUnit;
    type Error = Error;

    fn translate(&mut self, source: &P) -> Result<Self::Target, Self::Error> {
        let ir = fs::read_to_string(source).map_err(Error::IoError)?;
        let ir = ir_parse::translation_unit(&ir).map_err(Error::ParseError)?;
        Ok(ir)
    }
}
