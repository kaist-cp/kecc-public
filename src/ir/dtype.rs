use core::convert::TryFrom;
use core::fmt;
use core::ops::Deref;
use failure::Fail;
use lang_c::ast;
use lang_c::span::Node;
use std::collections::HashMap;
use std::hash::Hash;

use crate::ir::*;

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
    size_modifiers: Option<ast::TypeSpecifier>,
    signed_option: Option<ast::TypeSpecifier>,
    typedef_name: Option<String>,
    is_const: bool,
    is_typedef: bool,
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
    Array {
        inner: Box<Dtype>,
        size: usize,
    },
    Function {
        ret: Box<Dtype>,
        params: Vec<Dtype>,
    },
    Typedef {
        name: String,
        is_const: bool,
    },
}

impl BaseDtype {
    /// Apply `StorageClassSpecifier` to `BaseDtype`
    ///
    /// let's say declaration is `typedef int i32_t;`, if `self` represents `int`
    /// and `type_qualifier` represents `typedef`, `self` is transformed to
    /// representing `typedef int` after function performs.
    ///
    /// # Arguments
    ///
    /// * `self` - Part that has been converted to 'BaseDtype' on the declaration
    /// * `storage_class` - storage class requiring apply to 'self' immediately
    ///
    #[inline]
    fn apply_storage_class(
        &mut self,
        storage_class: &ast::StorageClassSpecifier,
    ) -> Result<(), DtypeError> {
        match storage_class {
            ast::StorageClassSpecifier::Typedef => {
                // duplicate `typedef` is allowed
                self.is_typedef = true;
            }
            _ => panic!("unsupported storage class"),
        }

        Ok(())
    }

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
            | ast::TypeSpecifier::Bool
            | ast::TypeSpecifier::Char
            | ast::TypeSpecifier::Int
            | ast::TypeSpecifier::Float
            | ast::TypeSpecifier::Double => {
                if self.scalar.is_some() {
                    return Err(DtypeError::Misc {
                        message: "two or more scalar types in declaration specifiers".to_string(),
                    });
                }
                self.scalar = Some(type_specifier.clone());
            }
            ast::TypeSpecifier::Short | ast::TypeSpecifier::Long => {
                if self.size_modifiers.is_some() {
                    return Err(DtypeError::Misc {
                        message: "two or more size modifiers in declaration specifiers".to_string(),
                    });
                }
                self.size_modifiers = Some(type_specifier.clone());
            }
            ast::TypeSpecifier::TypedefName(identifier) => {
                if self.typedef_name.is_some() {
                    return Err(DtypeError::Misc {
                        message: "two or more typedef names in declaration specifiers".to_string(),
                    });
                }
                self.typedef_name = Some(identifier.node.name.clone());
            }
            _ => todo!("apply_type_specifier: support {:?}", type_specifier),
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
            ast::DeclarationSpecifier::StorageClass(storage_class) => {
                self.apply_storage_class(&storage_class.node)?
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
    /// and the result is `Dtype::Int{ width: 4, is_signed: false, is_const: ture }`
    fn try_from(spec: BaseDtype) -> Result<Self, DtypeError> {
        assert!(
            !(spec.scalar.is_none()
                && spec.size_modifiers.is_none()
                && spec.signed_option.is_none()
                && spec.typedef_name.is_none()
                && !spec.is_const),
            "BaseDtype is empty"
        );

        let mut dtype = if let Some(name) = spec.typedef_name {
            if spec.scalar.is_some() || spec.signed_option.is_some() {
                return Err(DtypeError::Misc {
                    message: "typedef' cannot be used with scalar type or signed option"
                        .to_string(),
                });
            }

            Self::typedef(name)
        } else {
            // Creates `dtype` from scalar.
            let mut dtype = if let Some(t) = spec.scalar {
                match t {
                    ast::TypeSpecifier::Void => Self::unit(),
                    ast::TypeSpecifier::Bool => Self::BOOL,
                    ast::TypeSpecifier::Char => Self::CHAR,
                    ast::TypeSpecifier::Int => Self::INT,
                    ast::TypeSpecifier::Float => Self::FLOAT,
                    ast::TypeSpecifier::Double => Self::DOUBLE,
                    ast::TypeSpecifier::Unsigned | ast::TypeSpecifier::Signed => {
                        panic!("Signed option to scalar is not supported")
                    }
                    _ => panic!("Dtype::try_from::<BaseDtype>: {:?} is not a scalar type", t),
                }
            } else {
                Self::default()
            };

            // Applies size modifier
            if let Some(size_modifiers) = spec.size_modifiers {
                if dtype != Self::INT {
                    return Err(DtypeError::Misc {
                        message: "size modifier can only be used with `int`".to_string(),
                    });
                }

                dtype = match size_modifiers {
                    ast::TypeSpecifier::Short => Self::SHORT,
                    ast::TypeSpecifier::Long => Self::LONG,
                    _ => panic!(
                        "Dtype::try_from::<BaseDtype>: {:?} is not a size modifier",
                        size_modifiers
                    ),
                }
            }

            // Applies signedness.
            if let Some(signed_option) = spec.signed_option {
                let is_signed = match signed_option {
                    ast::TypeSpecifier::Signed => true,
                    ast::TypeSpecifier::Unsigned => false,
                    _ => panic!(
                        "Dtype::try_from::<BaseDtype>: {:?} is not a signed option",
                        signed_option
                    ),
                };

                dtype = dtype.set_signed(is_signed);
            }

            dtype
        };

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

            // A function call with an array argument performs array-to-pointer conversion.
            // For this reason, when `declarator` is from function parameter declaration
            // and `base_dtype` is `Dtype::Array`, `base_dtype` is transformed to pointer type.
            // https://www.eskimo.com/~scs/cclass/notes/sx10f.html
            if let Some(inner) = dtype.get_array_inner() {
                dtype = Self::pointer(inner.clone());
            }
        }
        Ok(dtype)
    }
}

impl Dtype {
    pub const BITS_OF_BYTE: usize = 8;
    pub const SIZE_OF_BYTE: usize = 1;
    // TODO: consider architecture dependency in the future
    pub const SIZE_OF_POINTER: usize = 4;

    pub const SIZE_OF_CHAR: usize = 1;
    pub const SIZE_OF_SHORT: usize = 2;
    pub const SIZE_OF_INT: usize = 4;
    pub const SIZE_OF_LONG: usize = 8;
    pub const SIZE_OF_LONGLONG: usize = 8;

    pub const SIZE_OF_FLOAT: usize = 4;
    pub const SIZE_OF_DOUBLE: usize = 8;

    // signed option cannot be applied to boolean value
    pub const BOOL: Self = Self::Int {
        width: 1,
        is_signed: false,
        is_const: false,
    };
    pub const CHAR: Self = Self::int(Self::SIZE_OF_CHAR * Self::BITS_OF_BYTE);
    pub const SHORT: Self = Self::int(Self::SIZE_OF_SHORT * Self::BITS_OF_BYTE);
    pub const INT: Self = Self::int(Self::SIZE_OF_INT * Self::BITS_OF_BYTE);
    pub const LONG: Self = Self::int(Self::SIZE_OF_LONG * Self::BITS_OF_BYTE);
    pub const LONGLONG: Self = Self::int(Self::SIZE_OF_LONGLONG * Self::BITS_OF_BYTE);

    pub const FLOAT: Self = Self::float(Self::SIZE_OF_FLOAT * Self::BITS_OF_BYTE);
    pub const DOUBLE: Self = Self::float(Self::SIZE_OF_DOUBLE * Self::BITS_OF_BYTE);

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

    // Suppose the C declaration is `int *a[2][3]`. Then `a`'s `ir::Dtype` should be `[2 x [3 x int*]]`.
    // But in the AST, it is parsed as `Array(3, Array(2, Pointer(int)))`, reversing the order of `2` and `3`.
    // In the recursive translation of declaration into Dtype, we need to insert `3` inside `[2 * int*]`.
    pub fn array(base_dtype: Dtype, size: usize) -> Self {
        match base_dtype {
            Self::Array {
                inner,
                size: old_size,
            } => {
                let inner = inner.deref().clone();
                let inner = Self::array(inner, size);
                Self::Array {
                    inner: Box::new(inner),
                    size: old_size,
                }
            }
            Self::Function { .. } => panic!("array size cannot be applied to function type"),
            inner => Self::Array {
                inner: Box::new(inner),
                size,
            },
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
    pub fn typedef(name: String) -> Self {
        Self::Typedef {
            name,
            is_const: false,
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
    pub fn get_array_inner(&self) -> Option<&Dtype> {
        if let Self::Array { inner, .. } = self {
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
            Self::Array { .. } => true,
            Self::Function { .. } => true,
            Self::Typedef { .. } => panic!("typedef should be replaced by real dtype"),
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
            Self::Array { .. } => self,
            Self::Function { .. } => self,
            Self::Typedef { name, .. } => Self::Typedef { name, is_const },
        }
    }

    pub fn size_align_of(&self) -> Result<(usize, usize), DtypeError> {
        match self {
            Self::Unit { .. } => Ok((0, 1)),
            Self::Int { width, .. } | Self::Float { width, .. } => {
                let size_of = (*width + Self::BITS_OF_BYTE - 1) / Self::BITS_OF_BYTE;
                let align_of = size_of;

                Ok((size_of, align_of))
            }
            Self::Pointer { .. } => Ok((Self::SIZE_OF_POINTER, Self::SIZE_OF_POINTER)),
            Self::Array { inner, size, .. } => {
                let (size_of_inner, align_of_inner) = inner.size_align_of()?;

                Ok((
                    size * std::cmp::max(size_of_inner, align_of_inner),
                    align_of_inner,
                ))
            }
            Self::Function { .. } => Ok((0, 1)),
            Self::Typedef { .. } => panic!("typedef should be replaced by real dtype"),
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
    ) -> Result<(Self, bool), DtypeError> {
        let mut spec = BaseDtype::default();
        BaseDtype::apply_declaration_specifiers(&mut spec, specifiers)?;
        let is_typedef = spec.is_typedef;
        let dtype = Self::try_from(spec)?;

        Ok((dtype, is_typedef))
    }

    /// Generate `Dtype` based on declarator and `self` which has scalar type.
    ///
    /// let's say declaration is `const int * const * const a;`.
    /// In general `self` start with `const int` which has scalar type and
    /// `declarator` represents `* const * const` with `ast::Declarator`
    ///
    /// # Arguments
    ///
    /// * `declarator` - Parts requiring conversion to 'Dtype' on the declaration
    /// * `decl_spec`  - Containing information that should be referenced
    ///                  when creating `Dtype` from `Declarator`.
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
                ast::DerivedDeclarator::Array(array_decl) => {
                    assert!(array_decl.node.qualifiers.is_empty());
                    self.with_ast_array_size(&array_decl.node.size)?
                }
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

    /// Generates `Dtype` based on declarator and `self` which has scalar type.
    ///
    /// Let's say the AST declaration is `int a[2][3]`; `self` represents `int [2]`; and
    /// `array_size` is `[3]`. Then this function should return `int [2][3]`.
    ///
    /// # Arguments
    ///
    /// * `array_size` - the array size to add to the dtype `self`
    ///
    pub fn with_ast_array_size(self, array_size: &ast::ArraySize) -> Result<Self, DtypeError> {
        let expr = if let ast::ArraySize::VariableExpression(expr) = array_size {
            &expr.node
        } else {
            panic!("`ArraySize` is unsupported except `ArraySize::VariableExpression`")
        };

        let constant = Constant::try_from(expr)
            .expect("expression of `ArraySize::VariableExpression` must be constant value");

        let (value, _, is_signed) = constant.get_int().ok_or_else(|| DtypeError::Misc {
            message: "expression is not an integer constant expression".to_string(),
        })?;

        if is_signed && (value as i128) < 0 {
            return Err(DtypeError::Misc {
                message: "declared as an array with a negative size".to_string(),
            });
        }

        Ok(Self::array(self, value as usize))
    }

    pub fn resolve_typedefs(self, typedefs: &HashMap<String, Dtype>) -> Result<Self, DtypeError> {
        let dtype = match &self {
            Self::Unit { .. } | Self::Int { .. } | Self::Float { .. } => self,
            Self::Pointer { inner, is_const } => {
                let inner = inner.deref().clone().resolve_typedefs(typedefs)?;
                Dtype::pointer(inner).set_const(*is_const)
            }
            Self::Array { inner, size } => {
                let inner = inner.deref().clone().resolve_typedefs(typedefs)?;
                Dtype::Array {
                    inner: Box::new(inner),
                    size: *size,
                }
            }
            Self::Function { ret, params } => {
                let ret = ret.deref().clone().resolve_typedefs(typedefs)?;
                let params = params
                    .iter()
                    .map(|p| p.clone().resolve_typedefs(typedefs))
                    .collect::<Result<Vec<_>, _>>()?;

                Dtype::function(ret, params)
            }
            Self::Typedef { name, is_const } => {
                let dtype = typedefs
                    .get(name)
                    .ok_or_else(|| DtypeError::Misc {
                        message: format!("unknown type name `{}`", name),
                    })?
                    .clone();
                let is_const = dtype.is_const() || *is_const;

                dtype.set_const(is_const)
            }
        };

        Ok(dtype)
    }

    pub fn merge(self, other: Self) -> Result<Self, DtypeError> {
        if self == other {
            Ok(self)
        } else {
            Err(DtypeError::Misc {
                message: format!("Dtype::merge({:?}, {:?}) failed", self, other),
            })
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
                write!(f, "{}*{}", inner, if *is_const { "const" } else { "" })
            }
            Self::Array { inner, size, .. } => write!(f, "[{} x {}]", size, inner,),
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
            Self::Typedef { name, is_const } => {
                write!(f, "{}{}", if *is_const { "const " } else { "" }, name)
            }
        }
    }
}

impl Default for Dtype {
    fn default() -> Self {
        // default dtype is `int`(i32)
        Self::INT
    }
}
