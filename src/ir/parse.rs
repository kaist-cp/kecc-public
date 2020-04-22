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
                let mut decls = HashMap::new();
                for decl in ds {
                    let result = decls.insert(decl.name.unwrap(), decl.inner);
                    assert!(result.is_none());
                }
                TranslationUnit { decls }
            }

        rule named_decl() -> Named<Declaration> =
            "var" __ dtype:dtype() __ var:global_variable() _ "=" _ initializer:initializer() {
                Named::new(Some(var), Declaration::Variable {
                    dtype: dtype,
                    initializer,
                })
            }
        /
            "fun" __ dtype:dtype() __ var:global_variable() _ "{" _ fun_body:fun_body() _ "}" {
                Named::new(Some(var), Declaration::Function {
                    signature: FunctionSignature::new(Dtype::function(Dtype::int(32), Vec::new())),
                    definition: Some(fun_body),
                })
            }
        /
            "fun" __ dtype:dtype() __ var:global_variable() {
                Named::new(Some(var), Declaration::Function {
                    signature: FunctionSignature::new(Dtype::function(Dtype::int(32), Vec::new())),
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
            "*" _ inner:dtype() { Dtype::pointer(inner) }
        / expected!("dtype")

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
                (bid, number, Named::new(name, instruction))
            }
        / expected!("instruction")

        rule instruction_inner() -> Instruction =
            "call" __ callee:operand() _ "(" _ args:(operand() ** (_ "," _)) _ ")" {
                Instruction::Call {
                    callee,
                    args,
                    return_type: Dtype::unit(), // TODO
                }
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
            "typecast" __ value:operand() __ "to" __ target_dtype:dtype() {
                Instruction::TypeCast { value, target_dtype }
            }
        /
            "minus" __ operand:operand() {
                let dtype = operand.dtype();
                Instruction::UnaryOp {
                    op: UnaryOperator::Minus,
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
            "<instruction>" {
                todo!()
            }
        / expected!("instruction_inner")

        rule arith_op() -> BinaryOperator =
            "add" { BinaryOperator::Plus }
        /
            "sub" { BinaryOperator::Minus }

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

        rule exit() -> BlockExit =
            "j" __ arg:jump_arg() {
                BlockExit::Jump { arg }
            }
        /
            "br" __ condition:operand() __ arg_then:jump_arg() __ arg_else:jump_arg() {
                BlockExit::ConditionalJump { condition, arg_then, arg_else }
            }
        /
            "switch" __ value:operand() __ "default" __ default:jump_arg() _ "[" _ cases:(switch_case() ** __) _ "]" {
                BlockExit::Switch { value, default, cases }
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
            n:number() {
                Constant::int(n as _, Dtype::int(128)) // TODO: the right dtype
            }
        /
            "undef" {
                Constant::undef(Dtype::unit()) // TODO
            }
        /
            "unit" {
                Constant::undef(Dtype::unit()) // TODO
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
            name:global_variable() {
                Operand::Constant(Constant::GlobalVariable {
                    name,
                    dtype: Dtype::unit(), // TODO
                })
            }
        /
            constant:constant() ":" dtype:dtype() {
                let constant = match (&constant, &dtype) {
                    (Constant::Int { value, .. }, Dtype::Int { width, is_signed, .. }) => {
                        Constant::Int {
                            value: *value,
                            width: *width,
                            is_signed: *is_signed,
                        }
                    }
                    (Constant::Undef { .. }, _) => {
                        Constant::undef(dtype.clone())
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
