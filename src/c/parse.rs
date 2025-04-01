use core::ops::Deref;
use std::path::Path;

use lang_c::ast::*;
use lang_c::driver::{Config, Error as ParseError, parse};
use lang_c::span::Node;

use crate::Translate;
use crate::utils::AssertSupported;
use crate::utils::NotSupportedErr;

/// Parse Error
#[derive(Debug)]
pub enum Error {
    ParseError(ParseError),
    Unsupported(NotSupportedErr),
}

/// C file Parser.
#[derive(Default, Clone, Copy, Debug)]
pub struct Parse;

impl<P: AsRef<Path>> Translate<P> for Parse {
    type Target = TranslationUnit;
    type Error = Error;

    fn translate(&mut self, source: &P) -> Result<Self::Target, Self::Error> {
        let config = Config::default();
        let ast = parse(&config, source).map_err(Error::ParseError)?;
        let unit = ast.unit;

        unit.assert_supported().map_err(Error::Unsupported)?;
        Ok(unit)
    }
}

impl<T: AssertSupported> AssertSupported for Node<T> {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        self.node.assert_supported()
    }
}

impl<T: AssertSupported> AssertSupported for Option<T> {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        if let Some(this) = self {
            this.assert_supported()?;
        }

        Ok(())
    }
}

impl<T: AssertSupported> AssertSupported for Box<T> {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        self.deref().assert_supported()
    }
}

impl<T: AssertSupported> AssertSupported for Vec<T> {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        self.iter().try_for_each(|v| v.assert_supported())
    }
}

impl<T: AssertSupported> AssertSupported for [T] {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        self.iter().try_for_each(|v| v.assert_supported())
    }
}

impl AssertSupported for TranslationUnit {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        self.0.assert_supported()
    }
}

impl AssertSupported for ExternalDeclaration {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        match self {
            Self::Declaration(decl) => {
                if !is_valid_global_variable_declaration(&decl.node) {
                    return Err(NotSupportedErr("invalid global declaration".into()));
                }

                decl.assert_supported()
            }
            Self::StaticAssert(_) => panic!("ExternalDeclaration::StaticAssert"),
            Self::FunctionDefinition(fdef) => fdef.assert_supported(),
        }
    }
}

impl AssertSupported for Declaration {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        self.specifiers.assert_supported()?;
        self.declarators.assert_supported()
    }
}

impl AssertSupported for FunctionDefinition {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        self.specifiers.assert_supported()?;
        self.declarator.assert_supported()?;
        assert!(self.declarations.is_empty());
        self.statement.assert_supported()
    }
}

impl AssertSupported for DeclarationSpecifier {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        match self {
            Self::StorageClass(storage_class) => storage_class.assert_supported(),
            Self::TypeSpecifier(type_specifier) => type_specifier.assert_supported(),
            Self::TypeQualifier(type_qualifier) => type_qualifier.assert_supported(),
            Self::Function(_) => panic!("DeclarationSpecifier::Function"),
            Self::Alignment(_) => panic!("DeclarationSpecifier::Alignment"),
            Self::Extension(_) => panic!("DeclarationSpecifier::Extension"),
        }
    }
}

impl AssertSupported for StorageClassSpecifier {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        if *self == Self::Typedef {
            Ok(())
        } else {
            Err(NotSupportedErr("".into()))
        }
    }
}

impl AssertSupported for TypeSpecifier {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        match self {
            Self::Void => Ok(()),
            Self::Char => Ok(()),
            Self::Short => Ok(()),
            Self::Int => Ok(()),
            Self::Long => Ok(()),
            Self::Float => Ok(()),
            Self::Double => Ok(()),
            Self::Signed => Ok(()),
            Self::Unsigned => Ok(()),
            Self::Bool => Ok(()),
            Self::Complex => panic!("TypeSpecifier::Complex"),
            Self::Atomic(_) => panic!("TypeSpecifier::Atomic"),
            Self::Struct(struct_type) => struct_type.assert_supported(),
            Self::Enum(_) => panic!("TypeSpecifier::Enum"),
            Self::TypedefName(_) => Ok(()),
            Self::TypeOf(_) => panic!("TypeSpecifier::TypeOf"),
            Self::TS18661Float(_) => panic!("TypeSpecifier::TS18661Float"),
        }
    }
}

impl AssertSupported for StructType {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        self.kind.assert_supported()?;
        self.declarations.assert_supported()
    }
}

impl AssertSupported for StructDeclaration {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        match self {
            Self::Field(field) => field.assert_supported(),
            Self::StaticAssert(_) => Err(NotSupportedErr("StructDeclaration::StaticAssert".into())),
        }
    }
}

impl AssertSupported for StructField {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        self.specifiers.assert_supported()?;
        self.declarators.assert_supported()
    }
}

impl AssertSupported for StructDeclarator {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        if self.bit_width.is_some() {
            return Err(NotSupportedErr("bitfield".into()));
        }
        self.declarator.assert_supported()
    }
}

impl AssertSupported for StructKind {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        match self {
            Self::Struct => Ok(()),
            Self::Union => Err(NotSupportedErr("StructKind::Union".into())),
        }
    }
}

impl AssertSupported for AlignmentSpecifier {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        match self {
            Self::Type(typename) => typename.assert_supported(),
            Self::Constant(_) => std::panic::panic_any(AlignmentSpecifier::Constant),
        }
    }
}

impl AssertSupported for InitDeclarator {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        self.declarator.assert_supported()?;
        self.initializer.assert_supported()
    }
}

impl AssertSupported for Initializer {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        match self {
            Self::Expression(expr) => expr.assert_supported(),
            Self::List(items) => items.assert_supported(),
        }
    }
}

impl AssertSupported for InitializerListItem {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        assert!(self.designation.is_empty());
        self.initializer.assert_supported()
    }
}

impl AssertSupported for Declarator {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        self.kind.assert_supported()?;
        assert!(self.extensions.is_empty());
        self.derived.assert_supported()
    }
}

impl AssertSupported for DerivedDeclarator {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        match self {
            Self::Pointer(pointer_qualifiers) => pointer_qualifiers.assert_supported(),
            Self::Array(array_decl) => array_decl.assert_supported(),
            Self::Function(func_decl) => func_decl.assert_supported(),
            // Support when K&R function has no parameter
            Self::KRFunction(kr_func_decl) => {
                if kr_func_decl.is_empty() {
                    Ok(())
                } else {
                    Err(NotSupportedErr("".into()))
                }
            }
            Self::Block(_) => Err(NotSupportedErr("DerivedDeclarator::Block".into())),
        }
    }
}

impl AssertSupported for PointerQualifier {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        match self {
            Self::TypeQualifier(type_qualifier) => type_qualifier.assert_supported(),
            Self::Extension(_) => Err(NotSupportedErr("PointerQualifier::Extension".into())),
        }
    }
}

impl AssertSupported for ArrayDeclarator {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        // In C99, type qualifier(e.g., const) is allowed when
        // array declarator is used as function parameter.
        // However, KECC does not allow this feature because
        // it complicates IR generating logic.
        assert!(self.qualifiers.is_empty());
        self.size.assert_supported()
    }
}

impl AssertSupported for TypeQualifier {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        match self {
            Self::Const => Ok(()),
            _ => panic!("TypeQualifier::_"),
        }
    }
}

impl AssertSupported for ArraySize {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        match self {
            Self::VariableExpression(expr) => expr.assert_supported(),
            _ => Err(NotSupportedErr("ArraySize::_".into())),
        }
    }
}

impl AssertSupported for FunctionDeclarator {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        if self.ellipsis != Ellipsis::None {
            return Err(NotSupportedErr("Ellipsis".into()));
        }
        self.parameters.assert_supported()
    }
}

impl AssertSupported for ParameterDeclaration {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        self.specifiers.assert_supported()?;
        self.declarator.assert_supported()?;
        if !self.extensions.is_empty() {
            return Err(NotSupportedErr("extensions".into()));
        }

        Ok(())
    }
}

impl AssertSupported for DeclaratorKind {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        match self {
            Self::Abstract => Ok(()),
            Self::Identifier(_) => Ok(()),
            Self::Declarator(decl) => decl.assert_supported(),
        }
    }
}

impl AssertSupported for BlockItem {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        match self {
            Self::Declaration(decl) => {
                decl.node.declarators.assert_supported()?;

                for spec in &decl.node.specifiers {
                    spec.assert_supported()?;
                    match &spec.node {
                        DeclarationSpecifier::StorageClass(_) => {
                            // In C, `typedef` can be declared within the function.
                            // However, KECC does not allow this feature
                            // because it complicates IR generating logic.
                            // For example, KECC does not allow a declaration using `typedef`
                            // such as `typedef int i32_t;` declaration in a function definition.
                            panic!("`StorageClassifier` is not allowed at `BlockItem`")
                        }
                        DeclarationSpecifier::TypeSpecifier(type_specifier) => {
                            if let TypeSpecifier::Struct(struct_type) = &type_specifier.node {
                                struct_type.node.kind.assert_supported()?;
                                // In C, `struct` can be declared within the function.
                                // However, KECC does not allow this feature
                                // because it complicates IR generating logic.
                                // For example, KECC allows `struct A var;` declaration
                                // using pre-declared `struct A`, but not `struct A { int a; } var;`
                                // which tries to declare `struct A` newly.
                                assert!(struct_type.node.declarations.is_none());
                            }
                        }
                        _ => (),
                    }
                }

                Ok(())
            }
            Self::StaticAssert(_) => panic!("BlockItem::StaticAssert"),
            Self::Statement(stmt) => stmt.assert_supported(),
        }
    }
}

impl AssertSupported for ForInitializer {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        match self {
            Self::Empty => Ok(()),
            Self::Expression(expr) => expr.assert_supported(),
            Self::Declaration(decl) => decl.assert_supported(),
            Self::StaticAssert(_) => panic!("ForInitializer::StaticAssert"),
        }
    }
}

impl AssertSupported for Statement {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        match self {
            Self::Labeled(_) => panic!("Statement::Labeled"),
            Self::Compound(items) => items.assert_supported(),
            Self::Expression(expr) => expr.assert_supported(),
            Self::If(stmt) => {
                stmt.node.condition.assert_supported()?;
                stmt.node.then_statement.assert_supported()?;
                stmt.node.else_statement.assert_supported()
            }
            Self::Switch(stmt) => stmt.assert_supported(),
            Self::While(stmt) => {
                stmt.node.expression.assert_supported()?;
                stmt.node.statement.assert_supported()
            }
            Self::DoWhile(stmt) => {
                stmt.node.statement.assert_supported()?;
                stmt.node.expression.assert_supported()
            }
            Self::For(stmt) => {
                stmt.node.initializer.assert_supported()?;
                stmt.node.condition.assert_supported()?;
                stmt.node.step.assert_supported()?;
                stmt.node.statement.assert_supported()
            }
            Self::Goto(_) => panic!("Statement::Goto"),
            Self::Continue | Self::Break => Ok(()),
            Self::Return(expr) => expr.assert_supported(),
            Self::Asm(_) => panic!("Statement::Asm"),
        }
    }
}

impl AssertSupported for SwitchStatement {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        self.expression.assert_supported()?;

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
                label_stmt.node.label.assert_supported()?;
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
                    BlockItem::Declaration(decl) => decl.assert_supported()?,
                    BlockItem::StaticAssert(_) => panic!("BlockItem::StaticAssert"),
                    BlockItem::Statement(stmt) => {
                        assert_ne!(
                            &stmt.node,
                            &Statement::Break,
                            "`BlockItem::Statement` in the `Statement::Compound` of the \
                             `label` should not be `Statement::Break` except the last one"
                        );
                        stmt.assert_supported()?;
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

        Ok(())
    }
}

impl AssertSupported for Expression {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        match self {
            Self::Identifier(_) => Ok(()),
            Self::Constant(constant) => constant.assert_supported(),
            Self::StringLiteral(_) => Ok(()),
            Self::GenericSelection(_) => panic!("Expression::GenericSelection"),
            Self::Member(member) => member.assert_supported(),
            Self::Call(call) => call.assert_supported(),
            Self::CompoundLiteral(_) => panic!("Expression::CompoundLiteral"),
            Self::SizeOfTy(size_of_ty) => size_of_ty.assert_supported(),
            Self::SizeOfVal(size_of_val) => size_of_val.assert_supported(),
            Self::AlignOf(align_of) => align_of.assert_supported(),
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
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        match self {
            Self::Identifier(_) => panic!("Label::Identifier"),
            Self::Case(_) => Ok(()),
            Self::CaseRange(_) => panic!("Label::CaseRange"),
            Self::Default => Ok(()),
        }
    }
}

impl AssertSupported for MemberExpression {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        self.expression.assert_supported()
    }
}

impl AssertSupported for CallExpression {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        self.callee.assert_supported()?;
        self.arguments.assert_supported()
    }
}

impl AssertSupported for TypeName {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        self.specifiers.assert_supported()?;
        self.declarator.assert_supported()
    }
}

impl AssertSupported for SpecifierQualifier {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        match self {
            Self::TypeSpecifier(type_specifier) => type_specifier.assert_supported(),
            Self::TypeQualifier(type_qualifier) => type_qualifier.assert_supported(),
            Self::Extension(_) => Err(NotSupportedErr("SpecifierQualifier::Extension".into())),
        }
    }
}

impl AssertSupported for UnaryOperatorExpression {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        self.operator.assert_supported()?;
        self.operand.assert_supported()
    }
}

impl AssertSupported for CastExpression {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        self.type_name.assert_supported()?;
        self.expression.assert_supported()
    }
}

impl AssertSupported for BinaryOperatorExpression {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        self.operator.assert_supported()?;
        self.lhs.assert_supported()?;
        self.rhs.assert_supported()
    }
}

impl AssertSupported for Constant {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        match self {
            Self::Integer(integer) => integer.assert_supported(),
            Self::Float(float) => float.assert_supported(),
            Self::Character(_) => Ok(()),
        }
    }
}

impl AssertSupported for Integer {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        if self.suffix.imaginary {
            return Err(NotSupportedErr("imaginary".into()));
        }
        Ok(())
    }
}

impl AssertSupported for Float {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        self.suffix.format.assert_supported()?;
        if self.suffix.imaginary {
            return Err(NotSupportedErr("imaginary".into()));
        }
        Ok(())
    }
}

impl AssertSupported for FloatFormat {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        match self {
            Self::Float => Ok(()),
            Self::Double => Ok(()),
            Self::LongDouble => Ok(()),
            Self::TS18661Format(_) => Err(NotSupportedErr("TS18861".into())),
        }
    }
}

impl AssertSupported for UnaryOperator {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        Ok(())
    }
}

impl AssertSupported for BinaryOperator {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        Ok(())
    }
}

impl AssertSupported for ConditionalExpression {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        self.condition.assert_supported()?;
        self.then_expression.assert_supported()?;
        self.else_expression.assert_supported()
    }
}

impl AssertSupported for SizeOfTy {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        self.0.assert_supported()
    }
}

impl AssertSupported for SizeOfVal {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        self.0.assert_supported()
    }
}

impl AssertSupported for AlignOf {
    fn assert_supported(&self) -> Result<(), NotSupportedErr> {
        self.0.assert_supported()
    }
}

#[inline]
fn is_valid_global_variable_declaration(decl: &Declaration) -> bool {
    let declarators = &decl.declarators;

    declarators.iter().all(|init_decl| {
        if let Some(initializer) = &init_decl.node.initializer {
            is_valid_global_variable_initializer(&initializer.node)
        } else {
            true
        }
    })
}

#[inline]
fn is_valid_global_variable_initializer(initializer: &Initializer) -> bool {
    match initializer {
        Initializer::Expression(expr) => match &expr.node {
            Expression::Constant(_) => true,
            Expression::UnaryOperator(unary) => {
                matches!(
                    &unary.node.operator.node,
                    UnaryOperator::Minus | UnaryOperator::Plus
                ) && matches!(&unary.node.operand.node, Expression::Constant(_))
            }
            _ => false,
        },
        Initializer::List(items) => items
            .iter()
            .all(|item| is_valid_global_variable_initializer(&item.node.initializer.node)),
    }
}
