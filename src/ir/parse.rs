use std::fs;
use std::path::Path;

use lang_c::*;

use crate::ir::*;
use crate::utils::AssertSupported;
use crate::Translate;
use crate::*;

peg::parser! {
    grammar ir_parse() for str {
        rule whitespace() = quiet!{[' ' | '\n' | '\t']}

        rule _() = whitespace()*

        rule __() = whitespace()+

        pub rule translation_unit() -> TranslationUnit
            = _ named_structs:(named_struct() ** __) _ ds:(named_decl() ** __) _ {
                let mut structs = HashMap::new();
                for named_struct in &named_structs {
                    let name = named_struct.name.as_ref().unwrap();
                    let struct_type = &named_struct.inner;
                    let result = structs.insert(name.clone(), struct_type.clone());
                    assert!(result.is_none());
                }

                // Resolve struct type in structs
                for named_struct in named_structs {
                    let name = named_struct.name.unwrap();
                    let dtype = some_or!(structs.get(&name).unwrap(), continue);
                    if dtype.get_struct_size_align_offsets().unwrap().is_none() {
                        resolve_structs(dtype.clone(), &mut structs);
                    }
                }

                let mut decls = BTreeMap::new();
                for decl in ds {
                    let result = decls.insert(decl.name.unwrap(), decl.inner);
                    assert!(result.is_none());
                }

                TranslationUnit { decls, structs }
            }

        rule named_struct() -> Named<Option<Dtype>> =
            "struct" __ id:id() _ ":" _ "opaque" {
                Named::new(Some(id), None)
            }
        /
            "struct" __ id:id() _ ":" _ "{" _ fields:(struct_field() ** (_ "," _)) _ "}"  {
                let struct_type = Dtype::structure(Some(id.clone()), Some(fields));
                Named::new(Some(id), Some(struct_type))
            }
        /
            "<named_struct>" {
                todo!()
            }

        rule struct_field() -> Named<Dtype> =
            "%anon" _ ":" _ dtype:dtype() {
                Named::new(None, dtype)
            }
        /
            id:id() _ ":" _ dtype:dtype() {
                Named::new(Some(id), dtype)
            }
        /
            "<struct_field>" {
                todo!()
            }

        rule named_decl() -> Named<Declaration> =
            "var" __ dtype:dtype() __ var:global_variable() _ "=" _ initializer:initializer() {
                Named::new(Some(var), Declaration::Variable {
                    dtype,
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
            inner:dtype_inner() is_consts:(is_const_of_pointer() ** _) {
                let mut inner = inner;
                for is_const in is_consts {
                    inner = Dtype::pointer(inner).set_const(is_const);
                }
                inner
            }

        rule dtype_inner() -> Dtype =
            "unit" { Dtype::unit() }
        /
            "u" n:number() { Dtype::int(n).set_signed(false) }
        /
            "i" n:number() { Dtype::int(n) }
        /
            "f" n:number() { Dtype::float(n) }
        /
            "[" _ n:number() __ "x" __ inner:dtype() _ "]" {
                Dtype::Array { inner: Box::new(inner), size: n }
            }
        /
            "[ret:" _ ret:dtype() __ "params:(" params:(dtype() ** (_ "," _)) _ ")]" {
                Dtype::Function { ret: Box::new(ret), params }
            }
        /
            "struct" __ id:id() {
                Dtype::structure(Some(id), None)
            }
        /
            "const" __ dtype:dtype_inner() { dtype.set_const(true) }
        /
            expected!("dtype")

        rule is_const_of_pointer() -> bool =
            _ "*" _ "const" { true }
        /
            _ "*" { false }

        rule id() -> String =
            n:$(['_' | 'a'..='z' | 'A'..='Z']['_' | 'a'..='z' | 'A'..='Z' | '0'..='9']*) {
                String::from(n)
            }
        /
            temp_id:$(['%']['t']['0'..='9']+) {
                String::from(temp_id)
            }
        /
            expected!("id")

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
                    Instruction::GetElementPtr { ptr, offset, dtype }
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
                    dtype: Dtype::unit(), // TODO
                }
            }
        /
            "<instruction>" {
                todo!()
            }
        / expected!("instruction_inner")

        rule arith_op() -> ast::BinaryOperator =
            "add" { ast::BinaryOperator::Plus }
        /
            "sub" { ast::BinaryOperator::Minus }
        /
            "mul" { ast::BinaryOperator::Multiply }
        /
            "div" { ast::BinaryOperator::Divide }
        /
            "mod" { ast::BinaryOperator::Modulo }

        rule shift_op() -> ast::BinaryOperator =
            "shl" { ast::BinaryOperator::ShiftLeft }
        /
            "shr" { ast::BinaryOperator::ShiftRight }

        rule comparison_op() -> ast::BinaryOperator =
            "eq" { ast::BinaryOperator::Equals }
        /
            "ne" { ast::BinaryOperator::NotEquals }
        /
            "lt" { ast::BinaryOperator::Less }
        /
            "le" { ast::BinaryOperator::LessOrEqual }
        /
            "gt" { ast::BinaryOperator::Greater }
        /
            "ge" { ast::BinaryOperator::GreaterOrEqual }

        rule bitwise_op() -> ast::BinaryOperator =
            "and" { ast::BinaryOperator::BitwiseAnd }
        /
            "xor" { ast::BinaryOperator::BitwiseXor }
        /
            "or" { ast::BinaryOperator::BitwiseOr }

        rule unary_op() -> ast::UnaryOperator =
            "plus" { ast::UnaryOperator::Plus }
        /
            "minus" { ast::UnaryOperator::Minus }
        /
            "negate" { ast::UnaryOperator::Negate }

        rule exit() -> BlockExit =
            "j" __ arg:jump_arg() {
                BlockExit::Jump { arg }
            }
        /
            "br" __ condition:operand() _ "," _ arg_then:jump_arg() _ "," _ arg_else:jump_arg() {
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
            f:float_number() {
                Constant::float(f, Dtype::float(64)) // TODO: the right dtype
            }
        /
            "-" f:float_number() {
                Constant::minus(Constant::float(f, Dtype::float(64))) // TODO: the right dtype
            }
        /
            n:number() {
                Constant::int(n as _, Dtype::int(128)) // TODO: the right dtype
            }
        /
            "-" n:number() {
                Constant::minus(Constant::int(n as _, Dtype::int(128))) // TODO: the right dtype
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

        rule initializer() -> Option<ast::Initializer> =
            "default" {
                None
            }
        /
            init:ast_initializer() {
                init.assert_supported();
                Some(init)
            }
        /
            "<initializer>" {
                todo!()
            }

        rule ast_initializer() -> ast::Initializer =
            expr:ast_expression() {
                let expr = Box::new(span::Node::new(expr, span::Span::none()));
                ast::Initializer::Expression(expr)
            }
        /
            "{" _ exprs:(ast_initializer() ** (_ "," _)) _ "}" {
                let list = exprs
                    .iter()
                    .map(|e| {
                        let initializer = Box::new(span::Node::new(e.clone(), span::Span::none()));
                        let item = ast::InitializerListItem{
                            designation: Vec::new(),
                            initializer,
                        };
                        span::Node::new(item, span::Span::none())
                    })
                    .collect();
                ast::Initializer::List(list)
            }
        /
            "<ast_initializer>" {
                todo!()
            }

        rule ast_expression() -> ast::Expression =
            constant:ast_constant() {
                let constant = Box::new(span::Node::new(constant, span::Span::none()));
                ast::Expression::Constant(constant)
            }
        /
            operator:ast_unaryop() _ "(" _ constant:ast_constant() _ ")" {
                let constant = Box::new(span::Node::new(constant, span::Span::none()));
                let expr = ast::Expression::Constant(constant);
                let operand = Box::new(span::Node::new(expr, span::Span::none()));

                let unary_expr = ast::UnaryOperatorExpression{
                    operator: span::Node::new(operator, span::Span::none()),
                    operand,
                };
                let unary_expr = Box::new(span::Node::new(unary_expr, span::Span::none()));

                ast::Expression::UnaryOperator(unary_expr)
            }
        /
            "<ast_expression>" {
                todo!()
            }

        rule ast_unaryop() -> ast::UnaryOperator =
            "+" {
                ast::UnaryOperator::Plus
            }
        /
            "-" {
                ast::UnaryOperator::Minus
            }
        /
            "<ast_unaryop>" {
                todo!()
            }

        rule ast_constant() -> ast::Constant =
            float:ast_float() {
                ast::Constant::Float(float)
            }
        /
            integer:ast_integer() {
                ast::Constant::Integer(integer)
            }
        /
            "<ast_constant>" {
                todo!()
            }

        rule ast_integer() -> ast::Integer =
            number:$(['1'..='9' | 'a'..='f' | 'A'..='F']['0'..='9' | 'a'..='f' | 'A'..='F']*) suffix:ast_integer_suffix() {
                ast::Integer {
                    base: ast::IntegerBase::Decimal,
                    number: Box::from(number),
                    suffix,
                }
            }
        /
            base:ast_integer_base() number:$(['0'..='9' | 'a'..='f' | 'A'..='F']+) suffix:ast_integer_suffix() {
                ast::Integer {
                    base,
                    number: Box::from(number),
                    suffix,
                }
            }
        /
            "0" suffix:ast_integer_suffix() {
                ast::Integer {
                    base: ast::IntegerBase::Decimal,
                    number: Box::from("0"),
                    suffix,
                }
            }
        /
            "<ast_integer>" {
                todo!()
            }

        rule ast_integer_base() -> ast::IntegerBase =
            ['0']['x' | 'X'] {
                ast::IntegerBase::Hexadecimal
            }
        /
            "0" {
                ast::IntegerBase::Octal
            }
        /
            "<ast_integer_base>" {
                todo!()
            }

        rule ast_integer_suffix() -> ast::IntegerSuffix =
            ['l' | 'L'] {
                ast::IntegerSuffix {
                    size: ast::IntegerSize::Long,
                    unsigned: false,
                    imaginary: false,
                }
            }
        /
            "" {
                ast::IntegerSuffix {
                    size: ast::IntegerSize::Int,
                    unsigned: false,
                    imaginary: false,
                }
            }

        rule ast_float() ->  ast::Float =
            number:$(['0'..='9']+['.']['0'..='9']*) suffix:ast_float_suffix() {
                ast::Float {
                    base: ast::FloatBase::Decimal,
                    number: Box::from(number),
                    suffix,
                }
            }
        /
            "<ast_float>" {
                todo!()
            }

        rule ast_float_suffix() -> ast::FloatSuffix =
            ['f' | 'F'] {
                ast::FloatSuffix {
                    format: ast::FloatFormat::Float,
                    imaginary: false,
                }
            }
        /
            "" {
                ast::FloatSuffix {
                    format: ast::FloatFormat::Double,
                    imaginary: false,
                }
            }
    }
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Parse(peg::error::ParseError<peg::str::LineCol>),
    Resolve,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Parse {}

impl<P: AsRef<Path>> Translate<P> for Parse {
    type Target = TranslationUnit;
    type Error = Error;

    fn translate(&mut self, source: &P) -> Result<Self::Target, Self::Error> {
        let ir = fs::read_to_string(source).map_err(Error::Io)?;
        ir_parse::translation_unit(&ir).map_err(Error::Parse)
    }
}

#[inline]
fn resolve_structs(struct_type: Dtype, structs: &mut HashMap<String, Option<Dtype>>) {
    let name = struct_type
        .get_struct_name()
        .expect("`struct_type` must be struct type")
        .as_ref()
        .expect("`struct_type` must have a name")
        .clone();
    let fields = struct_type
        .get_struct_fields()
        .expect("`struct_type` must be struct type")
        .as_ref()
        .expect("`struct_type` must have fields");

    for field in fields {
        if field.deref().get_struct_name().is_some() {
            let name = field
                .deref()
                .get_struct_name()
                .expect("`field` must be struct type")
                .as_ref()
                .expect("`field` must have a name");
            let field = structs
                .get(name)
                .expect("element matched with `name` must exist")
                .as_ref()
                .expect("element matched with `name` must exist");

            if field.get_struct_size_align_offsets().unwrap().is_none() {
                resolve_structs(field.clone(), structs);
            }
        }
    }

    let filled_struct = struct_type
        .fill_size_align_offsets_of_struct(structs)
        .expect("`struct_type` must be struct type");

    let result = structs.insert(name, Some(filled_struct));
    assert!(result.is_some());
}
