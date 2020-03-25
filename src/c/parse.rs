use core::ops::Deref;
use std::path::Path;

use lang_c::ast::*;
use lang_c::driver::{parse, Config, Error as ParseError};
use lang_c::span::Node;

use crate::Translate;

#[derive(Debug)]
pub enum Error {
    ParseError(ParseError),
    #[allow(dead_code)]
    Unsupported,
}

#[derive(Default)]
pub struct Parse {}

impl<P: AsRef<Path>> Translate<P> for Parse {
    type Target = TranslationUnit;
    type Error = Error;

    fn translate(&mut self, source: &P) -> Result<Self::Target, Self::Error> {
        let config = Config::default();
        let ast = parse(&config, source).map_err(Error::ParseError)?;
        let unit = ast.unit;

        unit.assert_supported();
        Ok(unit)
    }
}

trait AssertSupported {
    fn assert_supported(&self);
}

impl<T: AssertSupported> AssertSupported for Node<T> {
    fn assert_supported(&self) {
        self.node.assert_supported();
    }
}

impl<T: AssertSupported> AssertSupported for Option<T> {
    fn assert_supported(&self) {
        if let Some(this) = self {
            this.assert_supported();
        }
    }
}

impl<T: AssertSupported> AssertSupported for Box<T> {
    fn assert_supported(&self) {
        self.deref().assert_supported();
    }
}

impl<T: AssertSupported> AssertSupported for Vec<T> {
    fn assert_supported(&self) {
        self.iter().for_each(AssertSupported::assert_supported);
    }
}

impl<T: AssertSupported> AssertSupported for [T] {
    fn assert_supported(&self) {
        self.iter().for_each(AssertSupported::assert_supported);
    }
}

impl AssertSupported for TranslationUnit {
    fn assert_supported(&self) {
        self.0.assert_supported();
    }
}

impl AssertSupported for ExternalDeclaration {
    fn assert_supported(&self) {
        match self {
            Self::Declaration(decl) => decl.assert_supported(),
            Self::StaticAssert(_) => panic!("ExternalDeclaration::StaticAssert"),
            Self::FunctionDefinition(fdef) => fdef.assert_supported(),
        }
    }
}

impl AssertSupported for Declaration {
    fn assert_supported(&self) {
        self.specifiers.assert_supported();
        self.declarators.assert_supported();
    }
}

impl AssertSupported for FunctionDefinition {
    fn assert_supported(&self) {
        self.specifiers.assert_supported();
        self.declarator.assert_supported();
        assert!(self.declarations.is_empty());
        self.statement.assert_supported();
    }
}

impl AssertSupported for DeclarationSpecifier {
    fn assert_supported(&self) {
        match self {
            Self::StorageClass(storage_class_specifier) => {
                storage_class_specifier.assert_supported()
            }
            Self::TypeSpecifier(type_specifier) => type_specifier.assert_supported(),
            Self::TypeQualifier(type_qualifier) => type_qualifier.assert_supported(),
            Self::Function(_) => panic!("DeclarationSpecifier::Function"),
            Self::Alignment(_) => panic!("DeclarationSpecifier::Alignment"),
            Self::Extension(_) => panic!("DeclarationSpecifier::Extension"),
        }
    }
}

impl AssertSupported for StorageClassSpecifier {
    fn assert_supported(&self) {
        match self {
            Self::Typedef => (),
            _ => panic!("StorageClassifier other than Typedef"),
        }
    }
}

impl AssertSupported for TypeSpecifier {
    fn assert_supported(&self) {
        match self {
            Self::Void => (),
            Self::Char => (),
            Self::Short => (),
            Self::Int => (),
            Self::Long => (),
            Self::Float => (),
            Self::Double => (),
            Self::Signed => (),
            Self::Unsigned => (),
            Self::Bool => (),
            Self::Complex => panic!("TypeSpecifier::Complex"),
            Self::Atomic(_) => panic!("TypeSpecifier::Atomic"),
            Self::Struct(struct_type) => struct_type.assert_supported(),
            Self::Enum(_) => panic!("TypeSpecifier::Enum"),
            Self::TypedefName(_) => (),
            Self::TypeOf(_) => panic!("TypeSpecifier::TypeOf"),
            Self::TS18661Float(_) => panic!("TypeSpecifier::TS18661Float"),
        }
    }
}

impl AssertSupported for StructType {
    fn assert_supported(&self) {
        self.kind.assert_supported();
        self.declarations.assert_supported();
    }
}

impl AssertSupported for StructDeclaration {
    fn assert_supported(&self) {
        match self {
            Self::Field(field) => field.assert_supported(),
            Self::StaticAssert(_) => panic!("StructDeclaration::StaticAssert"),
        }
    }
}

impl AssertSupported for StructField {
    fn assert_supported(&self) {
        self.specifiers.assert_supported();
        self.declarators.assert_supported();
    }
}

impl AssertSupported for StructDeclarator {
    fn assert_supported(&self) {
        self.declarator.assert_supported();
        assert_eq!(true, self.bit_width.is_none());
    }
}

impl AssertSupported for StructKind {
    fn assert_supported(&self) {
        match self {
            Self::Struct => (),
            Self::Union => panic!("StructKind::Union"),
        }
    }
}

impl AssertSupported for AlignmentSpecifier {
    fn assert_supported(&self) {
        match self {
            Self::Type(typename) => typename.assert_supported(),
            Self::Constant(_) => panic!(AlignmentSpecifier::Constant),
        }
    }
}

impl AssertSupported for InitDeclarator {
    fn assert_supported(&self) {
        self.declarator.assert_supported();
        self.initializer.assert_supported();
    }
}

impl AssertSupported for Initializer {
    fn assert_supported(&self) {
        match self {
            Self::Expression(expr) => expr.assert_supported(),
            Self::List(_) => panic!("Initializer::List"),
        }
    }
}

impl AssertSupported for Declarator {
    fn assert_supported(&self) {
        self.kind.assert_supported();
        self.derived.assert_supported();
        self.extensions.is_empty();
    }
}

impl AssertSupported for DerivedDeclarator {
    fn assert_supported(&self) {
        match self {
            Self::Pointer(pointer_qualifiers) => pointer_qualifiers.assert_supported(),
            Self::Array(array_decl) => array_decl.assert_supported(),
            Self::Function(func_decl) => func_decl.assert_supported(),
            // Support when K&R function has no parameter
            Self::KRFunction(kr_func_decl) => assert_eq!(true, kr_func_decl.is_empty()),
        }
    }
}

impl AssertSupported for PointerQualifier {
    fn assert_supported(&self) {
        match self {
            Self::TypeQualifier(type_qualifier) => type_qualifier.assert_supported(),
            Self::Extension(_) => panic!("PointerQualifier::Extension"),
        }
    }
}

impl AssertSupported for ArrayDeclarator {
    fn assert_supported(&self) {
        self.qualifiers.assert_supported();
        self.size.assert_supported();
    }
}

impl AssertSupported for TypeQualifier {
    fn assert_supported(&self) {
        match self {
            Self::Const => (),
            _ => panic!("TypeQualifier::_"),
        }
    }
}

impl AssertSupported for ArraySize {
    fn assert_supported(&self) {
        match self {
            Self::VariableExpression(expr) => expr.assert_supported(),
            _ => panic!("ArraySize::_"),
        }
    }
}

impl AssertSupported for FunctionDeclarator {
    fn assert_supported(&self) {
        self.parameters.assert_supported();
        assert_eq!(self.ellipsis, Ellipsis::None);
    }
}

impl AssertSupported for ParameterDeclaration {
    fn assert_supported(&self) {
        self.specifiers.assert_supported();
        self.declarator.assert_supported();
        self.extensions.is_empty();
    }
}

impl AssertSupported for DeclaratorKind {
    fn assert_supported(&self) {
        match self {
            Self::Abstract => (),
            Self::Identifier(_) => (),
            Self::Declarator(decl) => decl.assert_supported(),
        }
    }
}

impl AssertSupported for BlockItem {
    fn assert_supported(&self) {
        match self {
            Self::Declaration(decl) => decl.assert_supported(),
            Self::StaticAssert(_) => panic!("BlockItem::StaticAssert"),
            Self::Statement(stmt) => stmt.assert_supported(),
        }
    }
}

impl AssertSupported for ForInitializer {
    fn assert_supported(&self) {
        match self {
            Self::Empty => (),
            Self::Expression(expr) => expr.assert_supported(),
            Self::Declaration(decl) => decl.assert_supported(),
            Self::StaticAssert(_) => panic!("ForInitializer::StaticAssert"),
        }
    }
}

impl AssertSupported for Statement {
    fn assert_supported(&self) {
        match self {
            Self::Labeled(_) => panic!("Statement::Labeled"),
            Self::Compound(items) => items.assert_supported(),
            Self::Expression(expr) => expr.assert_supported(),
            Self::If(stmt) => {
                stmt.node.condition.assert_supported();
                stmt.node.then_statement.assert_supported();
                stmt.node.else_statement.assert_supported();
            }
            Self::Switch(stmt) => stmt.assert_supported(),
            Self::While(stmt) => {
                stmt.node.expression.assert_supported();
                stmt.node.statement.assert_supported();
            }
            Self::DoWhile(stmt) => {
                stmt.node.statement.assert_supported();
                stmt.node.expression.assert_supported();
            }
            Self::For(stmt) => {
                stmt.node.initializer.assert_supported();
                stmt.node.condition.assert_supported();
                stmt.node.step.assert_supported();
                stmt.node.statement.assert_supported();
            }
            Self::Goto(_) => panic!("Statement::Goto"),
            Self::Continue | Self::Break => (),
            Self::Return(expr) => expr.assert_supported(),
            Self::Asm(_) => panic!("Statement::Asm"),
        }
    }
}

impl AssertSupported for SwitchStatement {
    fn assert_supported(&self) {
        self.expression.assert_supported();

        let items = if let Statement::Compound(items) = &self.statement.node {
            items
        } else {
            panic!("`Statement` in the `switch` is unsupported except `Statement::Compound`")
        };

        for item in items {
            let stmt = if let BlockItem::Statement(stmt) = &item.node {
                &stmt.node
            } else {
                panic!(
                    "`BlockItem` in the `Statement::Compound` of the `switch` \
                     is unsupported except `BlockItem::Statement`"
                )
            };

            let stmt_in_label = if let Statement::Labeled(label_stmt) = stmt {
                label_stmt.node.label.assert_supported();
                &label_stmt.node.statement.node
            } else {
                panic!(
                    "`BlockItem::Statement` in the `Statement::Compound` of the `switch` \
                     is unsupported except `Statement::Labeled`"
                )
            };

            let items = if let Statement::Compound(items) = stmt_in_label {
                items
            } else {
                panic!("`Statement` in the `label` is unsupported except `Statement::Compound`")
            };

            // Split last and all the rest of the elements of the `Compound` items
            let (last, items) = items
                .split_last()
                .unwrap_or_else(|| panic!("`Statement::Compound` has no item"));

            for item in items {
                match &item.node {
                    BlockItem::Declaration(decl) => decl.assert_supported(),
                    BlockItem::StaticAssert(_) => panic!("BlockItem::StaticAssert"),
                    BlockItem::Statement(stmt) => {
                        assert_ne!(
                            &stmt.node,
                            &Statement::Break,
                            "`BlockItem::Statement` in the `Statement::Compound` of the \
                             `label` should not be `Statement::Break` except the last one"
                        );
                        stmt.assert_supported();
                    }
                }
            }

            // The last element of the `items` must be `Statement::Break`
            let stmt = if let BlockItem::Statement(stmt) = &last.node {
                &stmt.node
            } else {
                panic!(
                    "`BlockItem` in the `Statement::Compound` of the `label` \
                     is unsupported except `BlockItem::Statement`"
                )
            };

            assert_eq!(
                stmt,
                &Statement::Break,
                "the last `BlockItem` in the `Statement::Compound` \
                 of the `label` must be `Statement::Break`"
            );
        }
    }
}

impl AssertSupported for Expression {
    fn assert_supported(&self) {
        match self {
            Self::Identifier(_) => (),
            Self::Constant(constant) => constant.assert_supported(),
            Self::StringLiteral(_) => panic!("Expression::StringLiteral"),
            Self::GenericSelection(_) => panic!("Expression::GenericSelection"),
            Self::Member(member) => member.assert_supported(),
            Self::Call(call) => call.assert_supported(),
            Self::CompoundLiteral(_) => panic!("Expression::CompoundLiteral"),
            Self::SizeOf(typename) => typename.assert_supported(),
            Self::AlignOf(typename) => typename.assert_supported(),
            Self::UnaryOperator(unary) => unary.assert_supported(),
            Self::Cast(cast) => cast.assert_supported(),
            Self::BinaryOperator(binary) => binary.assert_supported(),
            Self::Conditional(conditional) => conditional.assert_supported(),
            Self::Comma(exprs) => exprs.assert_supported(),
            Self::OffsetOf(_) => panic!("Expression::OffsetOf"),
            Self::VaArg(_) => panic!("Expression::VaArg"),
            Self::Statement(_) => panic!("Expression::Statement"),
        }
    }
}

impl AssertSupported for Label {
    fn assert_supported(&self) {
        match self {
            Self::Identifier(_) => panic!("Label::Identifier"),
            Self::Case(_) => (),
            Self::Default => (),
        }
    }
}

impl AssertSupported for MemberExpression {
    fn assert_supported(&self) {
        self.expression.assert_supported();
    }
}

impl AssertSupported for CallExpression {
    fn assert_supported(&self) {
        self.callee.assert_supported();
        self.arguments.assert_supported();
    }
}

impl AssertSupported for TypeName {
    fn assert_supported(&self) {
        self.specifiers.assert_supported();
        self.declarator.assert_supported();
    }
}

impl AssertSupported for SpecifierQualifier {
    fn assert_supported(&self) {
        match self {
            Self::TypeSpecifier(type_specifier) => type_specifier.assert_supported(),
            Self::TypeQualifier(type_qualifier) => type_qualifier.assert_supported(),
        }
    }
}

impl AssertSupported for UnaryOperatorExpression {
    fn assert_supported(&self) {
        self.operator.assert_supported();
        self.operand.assert_supported();
    }
}

impl AssertSupported for CastExpression {
    fn assert_supported(&self) {
        self.type_name.assert_supported();
        self.expression.assert_supported();
    }
}

impl AssertSupported for BinaryOperatorExpression {
    fn assert_supported(&self) {
        self.operator.assert_supported();
        self.lhs.assert_supported();
        self.rhs.assert_supported();
    }
}

impl AssertSupported for Constant {
    fn assert_supported(&self) {
        match self {
            Self::Integer(integer) => integer.assert_supported(),
            Self::Float(float) => float.assert_supported(),
            Self::Character(_) => (),
        }
    }
}

impl AssertSupported for Integer {
    fn assert_supported(&self) {
        assert_eq!(false, self.suffix.imaginary);
    }
}

impl AssertSupported for Float {
    fn assert_supported(&self) {
        assert_eq!(self.base, FloatBase::Decimal);
        self.suffix.format.assert_supported();
        assert_eq!(false, self.suffix.imaginary);
    }
}

impl AssertSupported for FloatFormat {
    fn assert_supported(&self) {
        match self {
            Self::Float => (),
            Self::Double => (),
            Self::LongDouble => (),
            Self::TS18661Format(_) => panic!("TS18861"),
        }
    }
}

impl AssertSupported for UnaryOperator {
    fn assert_supported(&self) {}
}

impl AssertSupported for BinaryOperator {
    fn assert_supported(&self) {}
}

impl AssertSupported for ConditionalExpression {
    fn assert_supported(&self) {
        self.condition.assert_supported();
        self.then_expression.assert_supported();
        self.else_expression.assert_supported();
    }
}
