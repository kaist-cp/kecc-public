use core::convert::TryFrom;
use core::fmt;
use core::ops::Deref;
use itertools::izip;
use lang_c::ast;
use lang_c::span::Node;
use std::hash::Hash;

use failure::Fail;

#[derive(Debug, PartialEq, Fail)]
pub enum DtypeError {
    /// For uncommon error
    #[fail(display = "{}", message)]
    Misc { message: String },
}

pub trait HasDtype {
    fn dtype(&self) -> Dtype;
}

#[derive(Default)]
struct BaseDtype {
    scalar: Option<ast::TypeSpecifier>,
    signed_option: Option<ast::TypeSpecifier>,
    is_const: bool,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Dtype {
    Unit {
        is_const: bool,
    },
    Int {
        width: usize,
        is_signed: bool,
        is_const: bool,
    },
    Float {
        width: usize,
        is_const: bool,
    },
    Pointer {
        inner: Box<Dtype>,
        is_const: bool,
    },
    Function {
        ret: Box<Dtype>,
        params: Vec<Dtype>,
    },
}

impl BaseDtype {
    /// Apply `TypeSpecifier` to `BaseDtype`
    ///
    /// let's say declaration is `const int a;`, if `self` represents `int`
    /// and `type_specifier` represents `const`, `self` is transformed to
    /// representing `const int` after function performs.
    ///
    /// # Arguments
    ///
    /// * `self` - Part that has been converted to 'BaseDtype' on the declaration
    /// * `type_qualifier` - type qualifiers requiring apply to 'self' immediately
    ///
    #[inline]
    fn apply_type_specifier(
        &mut self,
        type_specifier: &ast::TypeSpecifier,
    ) -> Result<(), DtypeError> {
        match type_specifier {
            ast::TypeSpecifier::Unsigned | ast::TypeSpecifier::Signed => {
                if self.signed_option.is_some() {
                    return Err(DtypeError::Misc {
                        message: "duplicate signed option".to_string(),
                    });
                }
                self.signed_option = Some(type_specifier.clone());
            }
            ast::TypeSpecifier::Void
            | ast::TypeSpecifier::Char
            | ast::TypeSpecifier::Int
            | ast::TypeSpecifier::Float => {
                if self.scalar.is_some() {
                    return Err(DtypeError::Misc {
                        message: "two or more scalar types in declaration specifiers".to_string(),
                    });
                }
                self.scalar = Some(type_specifier.clone());
            }
            _ => todo!("support more like `double` in the future"),
        }

        Ok(())
    }

    /// Apply `Typequalifier` to `BaseDtype`
    ///
    /// let's say declaration is `const int a;`, if `self` represents `int`
    /// and `type_qualifier` represents `const`, `self` is transformed to
    /// representing `const int` after function performs.
    ///
    /// # Arguments
    ///
    /// * `self` - Part that has been converted to 'BaseDtype' on the declaration
    /// * `type_qualifier` - type qualifiers requiring apply to 'self' immediately
    ///
    #[inline]
    fn apply_type_qualifier(
        &mut self,
        type_qualifier: &ast::TypeQualifier,
    ) -> Result<(), DtypeError> {
        match type_qualifier {
            ast::TypeQualifier::Const => {
                // duplicate `const` is allowed
                self.is_const = true;
            }
            _ => panic!("type qualifier is unsupported except `const`"),
        }

        Ok(())
    }

    pub fn apply_typename_specifier(
        &mut self,
        typename_specifier: &ast::SpecifierQualifier,
    ) -> Result<(), DtypeError> {
        match typename_specifier {
            ast::SpecifierQualifier::TypeSpecifier(type_specifier) => {
                self.apply_type_specifier(&type_specifier.node)?
            }
            ast::SpecifierQualifier::TypeQualifier(type_qualifier) => {
                self.apply_type_qualifier(&type_qualifier.node)?
            }
        }

        Ok(())
    }

    pub fn apply_declaration_specifier(
        &mut self,
        declaration_specifier: &ast::DeclarationSpecifier,
    ) -> Result<(), DtypeError> {
        match declaration_specifier {
            // TODO: `dtype` must be defined taking into account all specifier information.
            ast::DeclarationSpecifier::StorageClass(_storage_class_spec) => {
                todo!("analyze storage class specifier keyword to create correct `dtype`")
            }
            ast::DeclarationSpecifier::TypeSpecifier(type_specifier) => {
                self.apply_type_specifier(&type_specifier.node)?
            }
            ast::DeclarationSpecifier::TypeQualifier(type_qualifier) => {
                self.apply_type_qualifier(&type_qualifier.node)?
            }
            _ => panic!("is_unsupported"),
        }

        Ok(())
    }

    /// Apply `PointerQualifier` to `BaseDtype`
    ///
    /// let's say pointer declarator is `* const` of `const int * const a;`.
    /// If `self` represents nothing, and `pointer_qualifier` represents `const`
    /// between first and second asterisk, `self` is transformed to
    /// representing `const` after function performs. This information is used later
    /// when generating `Dtype`.
    ///
    /// # Arguments
    ///
    /// * `self` - Part that has been converted to 'BaseDtype' on the pointer declarator
    /// * `pointer_qualifier` - Pointer qualifiers requiring apply to 'BaseDtype' immediately
    ///
    pub fn apply_pointer_qualifier(
        &mut self,
        pointer_qualifier: &ast::PointerQualifier,
    ) -> Result<(), DtypeError> {
        match pointer_qualifier {
            ast::PointerQualifier::TypeQualifier(type_qualifier) => {
                self.apply_type_qualifier(&type_qualifier.node)?;
            }
            ast::PointerQualifier::Extension(_) => {
                panic!("ast::PointerQualifier::Extension is unsupported")
            }
        }

        Ok(())
    }

    pub fn apply_typename_specifiers(
        &mut self,
        typename_specifiers: &[Node<ast::SpecifierQualifier>],
    ) -> Result<(), DtypeError> {
        for ast_spec in typename_specifiers {
            self.apply_typename_specifier(&ast_spec.node)?;
        }

        Ok(())
    }

    pub fn apply_declaration_specifiers(
        &mut self,
        declaration_specifiers: &[Node<ast::DeclarationSpecifier>],
    ) -> Result<(), DtypeError> {
        for ast_spec in declaration_specifiers {
            self.apply_declaration_specifier(&ast_spec.node)?;
        }

        Ok(())
    }
}

impl TryFrom<BaseDtype> for Dtype {
    type Error = DtypeError;

    /// Derive a data type containing scalar type from specifiers.
    ///
    /// # Example
    ///
    /// For declaration is `const unsigned int * p`, `specifiers` is `const unsigned int`,
    /// and the result is `Dtype::Int{ width: 32, is_signed: false, is_const: ture }`
    fn try_from(spec: BaseDtype) -> Result<Self, DtypeError> {
        assert!(
            !(spec.scalar.is_none() && spec.signed_option.is_none() && !spec.is_const),
            "BaseDtype is empty"
        );

        // Creates `dtype` from scalar.
        let mut dtype = if let Some(t) = spec.scalar {
            match t {
                ast::TypeSpecifier::Void => Self::unit(),
                ast::TypeSpecifier::Unsigned | ast::TypeSpecifier::Signed => {
                    panic!("Signed option to scalar is not supported")
                }
                ast::TypeSpecifier::Bool => Self::BOOL,
                ast::TypeSpecifier::Char => Self::CHAR,
                ast::TypeSpecifier::Short => Self::SHORT,
                ast::TypeSpecifier::Int => Self::INT,
                ast::TypeSpecifier::Long => Self::LONG,
                ast::TypeSpecifier::Float => Self::FLOAT,
                ast::TypeSpecifier::Double => Self::DOUBLE,
                _ => panic!("Unsupported ast::TypeSpecifier"),
            }
        } else {
            Dtype::default()
        };

        // Applies signedness.
        if let Some(signed_option) = spec.signed_option {
            let is_signed = match signed_option {
                ast::TypeSpecifier::Signed => true,
                ast::TypeSpecifier::Unsigned => false,
                _ => panic!(
                    "`signed_option` must be `TypeSpecifier::Signed` or `TypeSpecifier::Unsigned`"
                ),
            };

            dtype = dtype.set_signed(is_signed);
        }

        // Applies constness.
        assert!(!dtype.is_const());
        dtype = dtype.set_const(spec.is_const);

        Ok(dtype)
    }
}

impl TryFrom<&ast::TypeName> for Dtype {
    type Error = DtypeError;

    /// Derive a data type from typename.
    fn try_from(type_name: &ast::TypeName) -> Result<Self, Self::Error> {
        let mut spec = BaseDtype::default();
        BaseDtype::apply_typename_specifiers(&mut spec, &type_name.specifiers)?;
        let mut dtype = Self::try_from(spec)?;

        if let Some(declarator) = &type_name.declarator {
            dtype = dtype.with_ast_declarator(&declarator.node)?;
        }
        Ok(dtype)
    }
}

impl TryFrom<&ast::ParameterDeclaration> for Dtype {
    type Error = DtypeError;

    /// Generate `Dtype` based on parameter declaration
    fn try_from(parameter_decl: &ast::ParameterDeclaration) -> Result<Self, Self::Error> {
        let mut spec = BaseDtype::default();
        BaseDtype::apply_declaration_specifiers(&mut spec, &parameter_decl.specifiers)?;
        let mut dtype = Self::try_from(spec)?;

        if let Some(declarator) = &parameter_decl.declarator {
            dtype = dtype.with_ast_declarator(&declarator.node)?;
        }
        Ok(dtype)
    }
}

impl Dtype {
    pub const BOOL: Self = Self::int(1);
    pub const CHAR: Self = Self::int(8);
    pub const SHORT: Self = Self::int(16);
    pub const INT: Self = Self::int(32);
    pub const LONG: Self = Self::int(64);
    pub const LONGLONG: Self = Self::int(64);

    pub const FLOAT: Self = Self::float(32);
    pub const DOUBLE: Self = Self::float(64);

    const WIDTH_OF_BYTE: usize = 8;
    // TODO: consider architecture dependency in the future
    const WIDTH_OF_POINTER: usize = 32;

    #[inline]
    pub const fn unit() -> Self {
        Self::Unit { is_const: false }
    }

    #[inline]
    pub const fn int(width: usize) -> Self {
        Self::Int {
            width,
            is_signed: true,
            is_const: false,
        }
    }

    #[inline]
    pub const fn float(width: usize) -> Self {
        Self::Float {
            width,
            is_const: false,
        }
    }

    #[inline]
    pub fn pointer(inner: Dtype) -> Self {
        Self::Pointer {
            inner: Box::new(inner),
            is_const: false,
        }
    }

    #[inline]
    pub fn function(ret: Dtype, params: Vec<Dtype>) -> Self {
        Self::Function {
            ret: Box::new(ret),
            params,
        }
    }

    #[inline]
    pub fn get_int_width(&self) -> Option<usize> {
        if let Self::Int { width, .. } = self {
            Some(*width)
        } else {
            None
        }
    }

    #[inline]
    pub fn get_float_width(&self) -> Option<usize> {
        if let Self::Float { width, .. } = self {
            Some(*width)
        } else {
            None
        }
    }

    #[inline]
    pub fn get_pointer_inner(&self) -> Option<&Dtype> {
        if let Self::Pointer { inner, .. } = self {
            Some(inner.deref())
        } else {
            None
        }
    }

    #[inline]
    pub fn get_function_inner(&self) -> Option<(&Dtype, &Vec<Dtype>)> {
        if let Self::Function { ret, params } = self {
            Some((ret.deref(), params))
        } else {
            None
        }
    }

    #[inline]
    pub fn is_scalar(&self) -> bool {
        match self {
            Self::Unit { .. } => todo!(),
            Self::Int { .. } => true,
            Self::Float { .. } => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_int_signed(&self) -> bool {
        match self {
            Self::Int { is_signed, .. } => *is_signed,
            _ => panic!("only `Dtype::Int` can be judged whether it is sigend"),
        }
    }

    #[inline]
    pub fn is_const(&self) -> bool {
        match self {
            Self::Unit { is_const } => *is_const,
            Self::Int { is_const, .. } => *is_const,
            Self::Float { is_const, .. } => *is_const,
            Self::Pointer { is_const, .. } => *is_const,
            Self::Function { .. } => {
                panic!("there should be no case that check whether `Function` is `const`")
            }
        }
    }

    pub fn set_const(self, is_const: bool) -> Self {
        match self {
            Self::Unit { .. } => Self::Unit { is_const },
            Self::Int {
                width, is_signed, ..
            } => Self::Int {
                width,
                is_signed,
                is_const,
            },
            Self::Float { width, .. } => Self::Float { width, is_const },
            Self::Pointer { inner, .. } => Self::Pointer { inner, is_const },
            Self::Function { .. } => panic!("`const` cannot be applied to `Dtype::Function`"),
        }
    }

    /// Return byte size of `Dtype`
    pub fn size_of(&self) -> Result<usize, DtypeError> {
        // TODO: consider complex type like array, structure in the future
        match self {
            Self::Unit { .. } => Ok(0),
            Self::Int { width, .. } => Ok(*width / Self::WIDTH_OF_BYTE),
            Self::Float { width, .. } => Ok(*width / Self::WIDTH_OF_BYTE),
            Self::Pointer { .. } => Ok(Self::WIDTH_OF_POINTER / Self::WIDTH_OF_BYTE),
            Self::Function { .. } => Err(DtypeError::Misc {
                message: "`sizeof` cannot be used with function types".to_string(),
            }),
        }
    }

    /// Return alignment requirements of `Dtype`
    pub fn align_of(&self) -> Result<usize, DtypeError> {
        // TODO: consider complex type like array, structure in the future
        // TODO: when considering complex type like a structure,
        // the calculation method should be different from `Dtype::size_of`.
        match self {
            Self::Unit { .. } => Ok(0),
            Self::Int { width, .. } => Ok(*width / Self::WIDTH_OF_BYTE),
            Self::Float { width, .. } => Ok(*width / Self::WIDTH_OF_BYTE),
            Self::Pointer { .. } => Ok(Self::WIDTH_OF_POINTER / Self::WIDTH_OF_BYTE),
            Self::Function { .. } => Err(DtypeError::Misc {
                message: "`alignof` cannot be used with function types".to_string(),
            }),
        }
    }

    pub fn set_signed(self, is_signed: bool) -> Self {
        match self {
            Self::Int {
                width, is_const, ..
            } => Self::Int {
                width,
                is_signed,
                is_const,
            },
            _ => panic!("`signed` and `unsigned` only be applied to `Dtype::Int`"),
        }
    }

    /// Derive a data type from declaration specifiers.
    pub fn try_from_ast_declaration_specifiers(
        specifiers: &[Node<ast::DeclarationSpecifier>],
    ) -> Result<Self, DtypeError> {
        let mut spec = BaseDtype::default();
        BaseDtype::apply_declaration_specifiers(&mut spec, specifiers)?;
        Self::try_from(spec)
    }

    /// Generate `Dtype` based on declarator and `base_dtype` which has scalar type.
    ///
    /// let's say declaration is `const int * const * const a;`.
    /// In general `base_dtype` start with `const int` which has scalar type and
    /// `declarator` represents `* const * const` with `ast::Declarator`
    ///
    /// # Arguments
    ///
    /// * `declarator` - Parts requiring conversion to 'Dtype' on the declaration
    /// * `base_dtype` - Part that has been converted to 'Dtype' on the declaration
    ///
    pub fn with_ast_declarator(mut self, declarator: &ast::Declarator) -> Result<Self, DtypeError> {
        for derived_decl in &declarator.derived {
            self = match &derived_decl.node {
                ast::DerivedDeclarator::Pointer(pointer_qualifiers) => {
                    let mut specifier = BaseDtype::default();
                    for qualifier in pointer_qualifiers {
                        specifier.apply_pointer_qualifier(&qualifier.node)?;
                    }
                    Self::pointer(self).set_const(specifier.is_const)
                }
                ast::DerivedDeclarator::Array(_array_decl) => todo!(),
                ast::DerivedDeclarator::Function(func_decl) => {
                    let params = func_decl
                        .node
                        .parameters
                        .iter()
                        .map(|p| Self::try_from(&p.node))
                        .collect::<Result<Vec<_>, _>>()?;
                    Self::function(self, params)
                }
                ast::DerivedDeclarator::KRFunction(kr_func_decl) => {
                    // K&R function is allowed only when it has no parameter
                    assert!(kr_func_decl.is_empty());
                    Self::function(self, Vec::new())
                }
            };
        }

        let declarator_kind = &declarator.kind;
        match &declarator_kind.node {
            ast::DeclaratorKind::Abstract => panic!("ast::DeclaratorKind::Abstract is unsupported"),
            ast::DeclaratorKind::Identifier(_) => Ok(self),
            ast::DeclaratorKind::Declarator(declarator) => {
                self.with_ast_declarator(&declarator.node)
            }
        }
    }

    /// Check whether type conflict exists between the two `Dtype` objects.
    ///
    /// let's say expression is `const int a = 0; int b = 0; int c = a + b`.
    /// Although `const int` of `a` and `int` of `b` looks different, `Plus`(+) operations between
    /// these two types are possible without any type-casting. There is no conflict between
    /// `const int` and `int`.
    ///
    /// However, only the outermost const is ignored.
    /// If check equivalence between `const int *const` and `int *`, result is false. Because
    /// the second `const` (means left most `const`) of the `const int *const` is missed in `int *`.
    /// By the way, outermost `const` (means right most `const`) is not a consideration here.
    pub fn is_compatible(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Unit { .. }, Self::Unit { .. })
            | (Self::Int { .. }, Self::Int { .. })
            | (Self::Float { .. }, Self::Float { .. })
            | (Self::Pointer { .. }, Self::Pointer { .. }) => {
                self.clone().set_const(false) == other.clone().set_const(false)
            }
            (
                Self::Function { ret, params },
                Self::Function {
                    ret: other_ret,
                    params: other_params,
                },
            ) => {
                ret == other_ret
                    && params.len() == other_params.len()
                    && izip!(params, other_params).all(|(l, r)| l.is_compatible(r))
            }
            _ => false,
        }
    }
}

impl fmt::Display for Dtype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unit { is_const } => write!(f, "{}unit", if *is_const { "const " } else { "" }),
            Self::Int {
                width,
                is_signed,
                is_const,
            } => write!(
                f,
                "{}{}{}",
                if *is_const { "const " } else { "" },
                if *is_signed { "i" } else { "u" },
                width
            ),
            Self::Float { width, is_const } => {
                write!(f, "{}f{}", if *is_const { "const " } else { "" }, width)
            }
            Self::Pointer { inner, is_const } => {
                write!(f, "{}* {}", inner, if *is_const { "const" } else { "" })
            }
            Self::Function { ret, params } => write!(
                f,
                "{} ({})",
                ret,
                params
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

impl Default for Dtype {
    fn default() -> Self {
        // default dtype is `int`(i32)
        Self::INT
    }
}
