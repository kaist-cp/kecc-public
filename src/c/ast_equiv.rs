#![allow(unused_variables)]

use lang_c::ast::*;
use lang_c::span::Node;

use core::ops::Deref;

use itertools::izip;

trait IsEquiv {
    fn is_equiv(&self, other: &Self) -> bool;
}

impl<T: IsEquiv> IsEquiv for Node<T> {
    fn is_equiv(&self, other: &Self) -> bool {
        self.node.is_equiv(&other.node)
    }
}

impl<T: IsEquiv> IsEquiv for Box<T> {
    fn is_equiv(&self, other: &Self) -> bool {
        self.deref().is_equiv(other.deref())
    }
}

impl<T: IsEquiv> IsEquiv for &T {
    fn is_equiv(&self, other: &Self) -> bool {
        (*self).is_equiv(*other)
    }
}

impl<T: IsEquiv> IsEquiv for Option<T> {
    fn is_equiv(&self, other: &Self) -> bool {
        match (self, other) {
            (Some(lhs), Some(rhs)) => lhs.is_equiv(rhs),
            (None, None) => true,
            _ => false,
        }
    }
}

impl<T: IsEquiv> IsEquiv for Vec<T> {
    fn is_equiv(&self, other: &Self) -> bool {
        self.len() == other.len() && izip!(self, other).all(|(lhs, rhs)| lhs.is_equiv(rhs))
    }
}

impl IsEquiv for TranslationUnit {
    fn is_equiv(&self, other: &Self) -> bool {
        self.0.is_equiv(&other.0)
    }
}

impl IsEquiv for ExternalDeclaration {
    fn is_equiv(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Declaration(decl), Self::Declaration(other_decl)) => decl.is_equiv(other_decl),
            (Self::FunctionDefinition(fdef), Self::FunctionDefinition(other_fdef)) => {
                fdef.is_equiv(other_fdef)
            }
            _ => false,
        }
    }
}

impl IsEquiv for Declaration {
    fn is_equiv(&self, other: &Self) -> bool {
        self.specifiers.is_equiv(&other.specifiers) && self.declarators.is_equiv(&other.declarators)
    }
}

impl IsEquiv for FunctionDefinition {
    fn is_equiv(&self, other: &Self) -> bool {
        self.specifiers.is_equiv(&other.specifiers)
            && self.declarator.is_equiv(&other.declarator)
            && self.declarations.is_equiv(&other.declarations)
            && self.statement.is_equiv(&other.statement)
    }
}

impl IsEquiv for InitDeclarator {
    fn is_equiv(&self, other: &Self) -> bool {
        self.declarator.is_equiv(&other.declarator) && self.initializer.is_equiv(&other.initializer)
    }
}

impl IsEquiv for Initializer {
    fn is_equiv(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Expression(expr), Self::Expression(other_expr)) => expr.is_equiv(other_expr),
            _ => false,
        }
    }
}

impl IsEquiv for Declarator {
    fn is_equiv(&self, other: &Self) -> bool {
        self.kind.is_equiv(&other.kind) && self.derived.is_equiv(&other.derived)
    }
}

impl IsEquiv for DeclaratorKind {
    fn is_equiv(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Abstract, Self::Abstract) => true,
            (Self::Identifier(identifier), Self::Identifier(other_identifier)) => {
                identifier.node.name == other_identifier.node.name
            }
            (Self::Declarator(decl), Self::Declarator(other_decl)) => decl.is_equiv(&other_decl),
            _ => false,
        }
    }
}

impl IsEquiv for DerivedDeclarator {
    fn is_equiv(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Pointer(pointer_qualifiers), Self::Pointer(other_pointer_qualifiers)) => {
                pointer_qualifiers.is_equiv(other_pointer_qualifiers)
            }
            (Self::Array(array_decl), Self::Array(other_array_decl)) => {
                let array_decl = &array_decl.node;
                let other_array_decl = &other_array_decl.node;

                array_decl.qualifiers.is_equiv(&other_array_decl.qualifiers)
                    && array_decl.size.is_equiv(&other_array_decl.size)
            }
            (Self::Function(func_decl), Self::Function(other_func_decl)) => {
                let params = &func_decl.node.parameters;
                let other_params = &other_func_decl.node.parameters;
                params.is_equiv(other_params)
            }
            (Self::KRFunction(kr_func_decl), Self::KRFunction(other_kr_func_decl)) => {
                kr_func_decl.is_equiv(&other_kr_func_decl)
            }
            _ => false,
        }
    }
}

impl IsEquiv for PointerQualifier {
    fn is_equiv(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::TypeQualifier(type_qualifier), Self::TypeQualifier(other_type_qualifier)) => {
                type_qualifier.is_equiv(other_type_qualifier)
            }
            _ => false,
        }
    }
}

impl IsEquiv for ArraySize {
    fn is_equiv(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Unknown, Self::Unknown) => true,
            (Self::VariableUnknown, Self::VariableUnknown) => true,
            (Self::VariableExpression(expr), Self::VariableExpression(other_expr)) => {
                expr.is_equiv(&other_expr)
            }
            (Self::StaticExpression(expr), Self::StaticExpression(other_expr)) => {
                expr.is_equiv(&other_expr)
            }
            _ => false,
        }
    }
}

impl IsEquiv for ParameterDeclaration {
    fn is_equiv(&self, other: &Self) -> bool {
        self.specifiers.is_equiv(&other.specifiers)
            && self
                .declarator
                .as_ref()
                .map(|d| &d.node)
                .is_equiv(&other.declarator.as_ref().map(|d| &d.node))
    }
}

impl IsEquiv for Statement {
    fn is_equiv(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Labeled(stmt), Self::Labeled(other_stmt)) => {
                stmt.node.label.is_equiv(&other_stmt.node.label)
                    && stmt.node.statement.is_equiv(&other_stmt.node.statement)
            }
            (Self::Compound(items), Self::Compound(other_items)) => items.is_equiv(other_items),
            (Self::Expression(expr), Self::Expression(other_expr)) => {
                expr.as_ref().is_equiv(&other_expr.as_ref())
            }
            (Self::If(stmt), Self::If(other_stmt)) => {
                let else_stmt = stmt.node.else_statement.as_ref();
                let other_else_stmt = other_stmt.node.else_statement.as_ref();
                stmt.node.condition.is_equiv(&other_stmt.node.condition)
                    && stmt
                        .node
                        .then_statement
                        .is_equiv(&other_stmt.node.then_statement)
                    && else_stmt.is_equiv(&other_else_stmt)
            }
            (Self::Switch(stmt), Self::Switch(other_stmt)) => {
                stmt.node.expression.is_equiv(&other_stmt.node.expression)
                    && stmt.node.statement.is_equiv(&other_stmt.node.statement)
            }
            (Self::While(stmt), Self::While(other_stmt)) => {
                stmt.node.expression.is_equiv(&other_stmt.node.expression)
                    && stmt.node.statement.is_equiv(&other_stmt.node.statement)
            }
            (Self::DoWhile(stmt), Self::DoWhile(other_stmt)) => {
                stmt.node.statement.is_equiv(&other_stmt.node.statement)
                    && stmt.node.expression.is_equiv(&other_stmt.node.expression)
            }
            (Self::For(stmt), Self::For(other_stmt)) => {
                stmt.node.initializer.is_equiv(&other_stmt.node.initializer)
                    && stmt
                        .node
                        .condition
                        .as_ref()
                        .is_equiv(&other_stmt.node.condition.as_ref())
                    && stmt
                        .node
                        .step
                        .as_ref()
                        .is_equiv(&other_stmt.node.step.as_ref())
                    && stmt.node.statement.is_equiv(&other_stmt.node.statement)
            }
            (Self::Goto(label), Self::Goto(other_label)) => label.is_equiv(other_label),
            (Self::Continue, Self::Continue) => true,
            (Self::Break, Self::Break) => true,
            (Self::Return(expr), Self::Return(other_expr)) => expr.is_equiv(other_expr),
            _ => false,
        }
    }
}

impl IsEquiv for Label {
    fn is_equiv(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Identifier(ident), Self::Identifier(other_ident)) => ident.is_equiv(other_ident),
            (Self::Case(expr), Self::Case(other_expr)) => expr.is_equiv(other_expr),
            (Self::Default, Self::Default) => true,
            _ => false,
        }
    }
}

impl IsEquiv for Identifier {
    fn is_equiv(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl IsEquiv for ForInitializer {
    fn is_equiv(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Empty, Self::Empty) => true,
            (Self::Expression(expr), Self::Expression(other_expr)) => expr.is_equiv(other_expr),
            (Self::Declaration(decl), Self::Declaration(other_decl)) => decl.is_equiv(other_decl),
            _ => false,
        }
    }
}

impl IsEquiv for Expression {
    fn is_equiv(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Identifier(identifier), Self::Identifier(other_identifier)) => {
                identifier.is_equiv(other_identifier)
            }
            (Self::Constant(constant), Self::Constant(other_constant)) => {
                constant.is_equiv(other_constant)
            }
            (Self::StringLiteral(other_string_lit), Self::StringLiteral(string_lit)) => {
                string_lit.is_equiv(other_string_lit)
            }
            (Self::Member(member), Self::Member(other_member)) => member.is_equiv(other_member),
            (Self::Call(call), Self::Call(other_call)) => call.is_equiv(other_call),
            (Self::SizeOf(typename), Self::SizeOf(other_typename)) => {
                typename.is_equiv(other_typename)
            }
            (Self::AlignOf(typename), Self::AlignOf(other_typename)) => {
                typename.is_equiv(other_typename)
            }
            (Self::UnaryOperator(unary), Self::UnaryOperator(other_unary)) => {
                unary.node.operator.is_equiv(&other_unary.node.operator)
                    && unary.node.operand.is_equiv(&other_unary.node.operand)
            }
            (Self::Cast(cast), Self::Cast(other_cast)) => {
                cast.node.type_name.is_equiv(&other_cast.node.type_name)
                    && cast.node.expression.is_equiv(&other_cast.node.expression)
            }
            (Self::BinaryOperator(binary), Self::BinaryOperator(other_binary)) => {
                binary.node.lhs.is_equiv(&other_binary.node.lhs)
                    && binary.node.operator.is_equiv(&other_binary.node.operator)
                    && binary.node.rhs.is_equiv(&other_binary.node.rhs)
            }
            (Self::Conditional(conditional), Self::Conditional(other_conditional)) => {
                conditional
                    .node
                    .condition
                    .is_equiv(&other_conditional.node.condition)
                    && conditional
                        .node
                        .then_expression
                        .is_equiv(&other_conditional.node.then_expression)
                    && conditional
                        .node
                        .else_expression
                        .is_equiv(&other_conditional.node.else_expression)
            }
            (Self::Comma(exprs), Self::Comma(other_exprs)) => {
                exprs.as_ref().is_equiv(other_exprs.as_ref())
            }
            _ => false,
        }
    }
}

impl IsEquiv for TypeName {
    fn is_equiv(&self, other: &Self) -> bool {
        self.specifiers.is_equiv(&other.specifiers) && self.declarator.is_equiv(&other.declarator)
    }
}

impl IsEquiv for SpecifierQualifier {
    fn is_equiv(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::TypeSpecifier(type_specifier), Self::TypeSpecifier(other_type_specifier)) => {
                type_specifier.is_equiv(other_type_specifier)
            }

            (Self::TypeQualifier(type_qualifier), Self::TypeQualifier(other_type_qualifier)) => {
                type_qualifier.is_equiv(other_type_qualifier)
            }
            _ => false,
        }
    }
}

impl IsEquiv for MemberExpression {
    fn is_equiv(&self, other: &Self) -> bool {
        self.expression.is_equiv(&other.expression)
            && self.operator.is_equiv(&other.operator)
            && self.identifier.is_equiv(&other.identifier)
    }
}

impl IsEquiv for MemberOperator {
    fn is_equiv(&self, other: &Self) -> bool {
        self == other
    }
}

impl IsEquiv for UnaryOperator {
    fn is_equiv(&self, other: &Self) -> bool {
        self == other
    }
}

impl IsEquiv for BinaryOperator {
    fn is_equiv(&self, other: &Self) -> bool {
        self == other
    }
}

impl IsEquiv for Constant {
    fn is_equiv(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Integer(integer), Self::Integer(other_integer)) => {
                integer.base.is_equiv(&other_integer.base)
                    && integer.number == other_integer.number
                    && integer.suffix.is_equiv(&other_integer.suffix)
            }
            (Self::Float(float), Self::Float(other_float)) => {
                float.base == other_float.base
                    && float.number == other_float.number
                    && float.suffix.is_equiv(&other_float.suffix)
            }
            (Self::Character(literal), Self::Character(other_literal)) => literal == other_literal,
            _ => false,
        }
    }
}

impl IsEquiv for IntegerBase {
    fn is_equiv(&self, other: &Self) -> bool {
        self == other
    }
}

impl IsEquiv for IntegerSuffix {
    fn is_equiv(&self, other: &Self) -> bool {
        self.unsigned == other.unsigned && self.size == other.size
    }
}

impl IsEquiv for FloatSuffix {
    fn is_equiv(&self, other: &Self) -> bool {
        self.imaginary == other.imaginary && self.format == other.format
    }
}

impl IsEquiv for StringLiteral {
    fn is_equiv(&self, other: &Self) -> bool {
        self == other
    }
}

impl IsEquiv for BlockItem {
    fn is_equiv(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Declaration(decl), Self::Declaration(other_decl)) => decl.is_equiv(other_decl),
            (Self::Statement(statement), Self::Statement(other_statement)) => {
                statement.is_equiv(other_statement)
            }
            _ => false,
        }
    }
}

impl IsEquiv for DeclarationSpecifier {
    fn is_equiv(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::StorageClass(storage_class_spec),
                Self::StorageClass(other_storage_class_spec),
            ) => storage_class_spec.is_equiv(other_storage_class_spec),
            (Self::TypeSpecifier(type_specifier), Self::TypeSpecifier(other_type_specifier)) => {
                type_specifier.is_equiv(other_type_specifier)
            }
            (Self::TypeQualifier(type_qualifier), Self::TypeQualifier(other_type_qualifier)) => {
                type_qualifier.is_equiv(other_type_qualifier)
            }
            _ => false,
        }
    }
}

impl IsEquiv for StorageClassSpecifier {
    fn is_equiv(&self, other: &Self) -> bool {
        self == other
    }
}

impl IsEquiv for TypeSpecifier {
    fn is_equiv(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Void, Self::Void) => true,
            (Self::Char, Self::Char) => true,
            (Self::Short, Self::Short) => true,
            (Self::Int, Self::Int) => true,
            (Self::Long, Self::Long) => true,
            (Self::Float, Self::Float) => true,
            (Self::Double, Self::Double) => true,
            (Self::Signed, Self::Signed) => true,
            (Self::Unsigned, Self::Unsigned) => true,
            (Self::Bool, Self::Bool) => true,
            (Self::Struct(struct_type), Self::Struct(other_struct_type)) => {
                struct_type.is_equiv(other_struct_type)
            }
            (Self::Enum(enum_type), Self::Enum(other_enum_type)) => {
                enum_type.is_equiv(other_enum_type)
            }
            (Self::TypedefName(identifier), Self::TypedefName(other_identifier)) => {
                identifier.is_equiv(other_identifier)
            }
            _ => false,
        }
    }
}

impl IsEquiv for StructType {
    fn is_equiv(&self, other: &Self) -> bool {
        self.declarations.is_equiv(&other.declarations)
            && self.kind.is_equiv(&other.kind)
            && self.identifier.is_equiv(&other.identifier)
    }
}

impl IsEquiv for StructKind {
    fn is_equiv(&self, other: &Self) -> bool {
        self == other
    }
}

impl IsEquiv for StructDeclaration {
    fn is_equiv(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Field(struct_field), Self::Field(other_struct_field)) => {
                struct_field.is_equiv(other_struct_field)
            }
            _ => false,
        }
    }
}

impl IsEquiv for StructField {
    fn is_equiv(&self, other: &Self) -> bool {
        self.specifiers.is_equiv(&other.specifiers) && self.declarators.is_equiv(&other.declarators)
    }
}

impl IsEquiv for StructDeclarator {
    fn is_equiv(&self, other: &Self) -> bool {
        self.declarator.is_equiv(&other.declarator) && self.bit_width.is_equiv(&other.bit_width)
    }
}

impl IsEquiv for EnumType {
    fn is_equiv(&self, other: &Self) -> bool {
        self.identifier.is_equiv(&other.identifier) && self.enumerators.is_equiv(&other.enumerators)
    }
}

impl IsEquiv for Enumerator {
    fn is_equiv(&self, other: &Self) -> bool {
        self.identifier.is_equiv(&other.identifier) && self.expression.is_equiv(&other.expression)
    }
}

impl IsEquiv for TypeQualifier {
    fn is_equiv(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Const, Self::Const) => true,
            _ => false,
        }
    }
}

impl IsEquiv for CallExpression {
    fn is_equiv(&self, other: &Self) -> bool {
        self.callee.is_equiv(&other.callee) && self.arguments.is_equiv(&other.arguments)
    }
}

pub fn assert_ast_equiv(lhs: &TranslationUnit, rhs: &TranslationUnit) {
    if !lhs.is_equiv(rhs) {
        panic!(
            r#"assertion failed: `(left.is_equiv(right))`
             left: `{:?}`,
            right: `{:?}`"#,
            lhs, rhs
        )
    }
}
