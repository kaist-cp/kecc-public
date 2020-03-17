use core::fmt;
use itertools::izip;
use lang_c::ast;
use lang_c::span::Node;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use failure::Fail;

pub trait HasDtype {
    fn dtype(&self) -> Dtype;
}

#[derive(Debug, PartialEq, Fail)]
pub enum DtypeError {
    /// For uncommon error
    #[fail(display = "{}", message)]
    Misc { message: String },
}

#[derive(Debug, PartialEq)]
pub struct TranslationUnit {
    pub decls: HashMap<String, Declaration>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Declaration {
    Variable {
        dtype: Dtype,
        initializer: Option<Constant>,
    },
    Function {
        signature: FunctionSignature,
        definition: Option<FunctionDefinition>,
    },
}

impl Declaration {
    /// Create an appropriate declaration according to `dtype`.
    ///
    /// # Example
    ///
    /// If `int g = 0;` is declared, `dtype` is
    /// `ir::Dtype::Int{ width:32, is_signed:true, is_const:false }`.
    /// In this case, `ir::Declaration::Variable{ dtype, initializer: Some(Constant::I32(1)) }`
    /// is generated.
    ///
    /// Conversely, if `int foo();` is declared, `dtype` is
    /// `ir::Dtype::Function{ret: Scalar(Int), params: []}`.
    /// Thus, in this case, `ir::Declaration::Function` is generated.
    pub fn from_dtype(dtype: Dtype) -> Result<Self, DtypeError> {
        match &dtype {
            Dtype::Unit { .. } => Err(DtypeError::Misc {
                message: "storage size of `void` isn't known".to_string(),
            }),
            Dtype::Int { .. } | Dtype::Float { .. } | Dtype::Pointer { .. } => {
                Ok(Declaration::Variable {
                    dtype,
                    initializer: None,
                })
            }
            Dtype::Function { .. } => Ok(Declaration::Function {
                signature: FunctionSignature::new(dtype),
                definition: None,
            }),
        }
    }

    pub fn get_variable(&self) -> Option<(&Dtype, &Option<Constant>)> {
        if let Self::Variable { dtype, initializer } = self {
            Some((dtype, initializer))
        } else {
            None
        }
    }

    pub fn get_function(&self) -> Option<(&FunctionSignature, &Option<FunctionDefinition>)> {
        if let Self::Function {
            signature,
            definition,
        } = self
        {
            Some((signature, definition))
        } else {
            None
        }
    }

    /// Check if type is conflicting for pre-declared one
    ///
    /// In case of `Variable`, need to check if the two types are exactly the same.
    /// On the other hand, in the case of `Function`, outermost `const` of return type and
    /// parameters one is not an issue of concern.
    pub fn is_compatible(&self, other: &Declaration) -> bool {
        match (self, other) {
            (Self::Variable { dtype, .. }, Self::Variable { dtype: other, .. }) => dtype == other,
            (
                Self::Function { signature, .. },
                Self::Function {
                    signature: other, ..
                },
            ) => signature.dtype().is_compatible(&other.dtype()),
            _ => false,
        }
    }
}

impl HasDtype for Declaration {
    fn dtype(&self) -> Dtype {
        match self {
            Self::Variable { dtype, .. } => dtype.clone(),
            Self::Function { signature, .. } => signature.dtype(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDefinition {
    /// Element that must allocate memory before a function can be executed
    pub allocations: Vec<Dtype>,
    pub blocks: HashMap<BlockId, Block>,
    pub bid_init: BlockId,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionSignature {
    pub ret: Dtype,
    pub params: Vec<Dtype>,
}

impl FunctionSignature {
    pub fn new(dtype: Dtype) -> Self {
        let (ret, params) = dtype
            .get_function_inner()
            .expect("function signature's dtype must be function type");
        Self {
            ret: ret.clone(),
            params: params.clone(),
        }
    }
}

impl HasDtype for FunctionSignature {
    fn dtype(&self) -> Dtype {
        Dtype::function(self.ret.clone(), self.params.clone())
    }
}

#[derive(Default)]
struct Specifier {
    scalar: Option<ast::TypeSpecifier>,
    signed_option: Option<ast::TypeSpecifier>,
    is_const: bool,
}

impl Specifier {
    #[inline]
    fn analyze_ast_type_specifiers(
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

    #[inline]
    fn analyze_ast_type_qualifier(
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

    pub fn from_ast_typename_specifier(
        &mut self,
        typename_specifier: &ast::SpecifierQualifier,
    ) -> Result<(), DtypeError> {
        match typename_specifier {
            ast::SpecifierQualifier::TypeSpecifier(type_specifier) => {
                self.analyze_ast_type_specifiers(&type_specifier.node)?
            }
            ast::SpecifierQualifier::TypeQualifier(type_qualifier) => {
                self.analyze_ast_type_qualifier(&type_qualifier.node)?
            }
        }

        Ok(())
    }

    pub fn from_ast_declaration_specifier(
        &mut self,
        declaration_specifier: &ast::DeclarationSpecifier,
    ) -> Result<(), DtypeError> {
        match declaration_specifier {
            // TODO: `dtype` must be defined taking into account all specifier information.
            ast::DeclarationSpecifier::StorageClass(_storage_class_spec) => {
                todo!("analyze storage class specifier keyword to create correct `dtype`")
            }
            ast::DeclarationSpecifier::TypeSpecifier(type_specifier) => {
                self.analyze_ast_type_specifiers(&type_specifier.node)?
            }
            ast::DeclarationSpecifier::TypeQualifier(type_qualifier) => {
                self.analyze_ast_type_qualifier(&type_qualifier.node)?
            }
            _ => panic!("is_unsupported"),
        }

        Ok(())
    }

    pub fn from_ast_typename_specifiers(
        typename_specifiers: &[Node<ast::SpecifierQualifier>],
    ) -> Result<Self, DtypeError> {
        let mut specifier = Specifier::default();

        for ast_spec in typename_specifiers {
            specifier.from_ast_typename_specifier(&ast_spec.node)?;
        }

        Ok(specifier)
    }

    pub fn from_ast_declaration_specifiers(
        declaration_specifiers: &[Node<ast::DeclarationSpecifier>],
    ) -> Result<Self, DtypeError> {
        let mut specifier = Specifier::default();

        for ast_spec in declaration_specifiers {
            specifier.from_ast_declaration_specifier(&ast_spec.node)?;
        }

        Ok(specifier)
    }
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
    pub fn unit() -> Self {
        Self::Unit { is_const: false }
    }

    #[inline]
    const fn int(width: usize) -> Self {
        Self::Int {
            width,
            is_signed: true,
            is_const: false,
        }
    }

    #[inline]
    const fn float(width: usize) -> Self {
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
    pub fn is_signed(&self) -> bool {
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

    /// Derive a data type containing scalar type from type specifier.
    pub fn from_ast_type_specifier(type_specifier: &ast::TypeSpecifier) -> Self {
        match type_specifier {
            ast::TypeSpecifier::Void => Self::unit(),
            ast::TypeSpecifier::Unsigned | ast::TypeSpecifier::Signed => {
                panic!("signed option to scalar is not supported")
            }
            ast::TypeSpecifier::Bool => Self::BOOL,
            ast::TypeSpecifier::Char => Self::CHAR,
            ast::TypeSpecifier::Short => Self::SHORT,
            ast::TypeSpecifier::Int => Self::INT,
            ast::TypeSpecifier::Long => Self::LONG,
            ast::TypeSpecifier::Float => Self::FLOAT,
            ast::TypeSpecifier::Double => Self::DOUBLE,
            _ => panic!("is unsupported"),
        }
    }

    /// Apply signed option to `Dtype`.
    pub fn apply_signed_option(
        self,
        signed_option: &ast::TypeSpecifier,
    ) -> Result<Self, DtypeError> {
        let is_signed = match signed_option {
            ast::TypeSpecifier::Signed => true,
            ast::TypeSpecifier::Unsigned => false,
            _ => panic!(
                "`signed_option` must be `TypeSpecifier::Signed` or `TypeSpecifier::Unsigned`"
            ),
        };

        match self {
            Self::Unit { .. } => Err(DtypeError::Misc {
                message: "`void` cannot matched with signed option".to_string(),
            }),
            Self::Int {
                width, is_const, ..
            } => Ok(Self::int(width).set_signed(is_signed).set_const(is_const)),
            Self::Float { .. } => Err(DtypeError::Misc {
                message: "floating point cannot matched with signed option".to_string(),
            }),
            Self::Pointer { .. } | Self::Function { .. } => {
                panic!("cannot apply signed option to `Self::Pointer` and `Self::Function`")
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

    /// Derive a data type from typename.
    pub fn from_ast_typename(type_name: &ast::TypeName) -> Result<Self, DtypeError> {
        let spec = Specifier::from_ast_typename_specifiers(&type_name.specifiers)?;
        let base_dtype = Self::from_ast_specifiers(spec)?;

        if let Some(declarator) = &type_name.declarator {
            Self::from_ast_declarator(&declarator.node, base_dtype)
        } else {
            Ok(base_dtype)
        }
    }

    /// Derive a data type from declaration specifiers.
    pub fn from_ast_declaration_specifiers(
        specifiers: &[Node<ast::DeclarationSpecifier>],
    ) -> Result<Self, DtypeError> {
        let spec = Specifier::from_ast_declaration_specifiers(specifiers)?;
        Self::from_ast_specifiers(spec)
    }

    /// Derive a data type containing scalar type from specifiers.
    ///
    /// # Example
    ///
    /// For declaration is `const unsigned int * p`, `specifiers` is `const unsigned int`,
    /// and the result is `Dtype::Int{ width: 32, is_signed: false, is_const: ture }`
    fn from_ast_specifiers(spec: Specifier) -> Result<Self, DtypeError> {
        // Generate appropriate `Dtype` object if `scalar` is `Some`
        let dtype = spec.scalar.map(|t| Dtype::from_ast_type_specifier(&t));

        // Update `dtype` obtained above or generate appropriate `Dtype` object if
        let dtype = match (spec.signed_option, dtype) {
            (Some(signed_option), Some(dtype)) => Some(dtype.apply_signed_option(&signed_option)?),
            (Some(signed_option), None) => {
                // If `signed_option` is used alone, it is used as`type_specifier`.
                // For example, `signed` can be replaced with `int` if it used alone.
                Some(Dtype::default().apply_signed_option(&signed_option)?)
            }
            (None, dtype) => dtype,
        };

        // Determining the final form of `dtype` according to the value of `is_const`
        let dtype = match (dtype, spec.is_const) {
            (Some(dtype), is_const) => {
                assert!(!dtype.is_const());
                dtype.set_const(is_const)
            }
            // If type specifier missing, defaults to `int`.
            // For example, `const` can be replaced with `const int` if it used alone.
            (None, true) => Dtype::default().set_const(true),
            (None, false) => {
                panic!("at least one valid declaration specifier is needed to generate `Dtype`")
            }
        };

        Ok(dtype)
    }

    /// Generate `Dtype` based on pointer qualifiers and `base_dtype` which has scalar type.
    ///
    /// let's say declaration is `const int * const * const a;`.
    /// If `base_dtype` represents `const int *`,
    /// `qualifiers` represents `const` between first and second asterisk.
    ///
    /// The important point here is that `qualifiers` exists between asterisks and asterisks or
    /// asterisks and identifiers.
    ///
    /// # Arguments
    ///
    /// * `qualifiers` - Pointer qualifiers requiring conversion to 'Dtype' immediately
    /// * `base_dtype` - Part that has been converted to 'Dtype' on the declaration
    ///
    pub fn from_ast_pointer_qualifiers(
        qualifiers: &[Node<ast::PointerQualifier>],
        base_dtype: Self,
    ) -> Result<Self, DtypeError> {
        let mut specifier = Specifier::default();

        for qualifier in qualifiers {
            match &qualifier.node {
                ast::PointerQualifier::TypeQualifier(type_qualifier) => {
                    specifier.analyze_ast_type_qualifier(&type_qualifier.node)?;
                }
                ast::PointerQualifier::Extension(_) => {
                    panic!("ast::PointerQualifier::Extension is unsupported")
                }
            }
        }

        Ok(base_dtype.set_const(specifier.is_const))
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
    pub fn from_ast_declarator(
        declarator: &ast::Declarator,
        base_dtype: Self,
    ) -> Result<Self, DtypeError> {
        let mut base_dtype = base_dtype;

        for derived_decl in &declarator.derived {
            base_dtype = match &derived_decl.node {
                ast::DerivedDeclarator::Pointer(pointer_qualifiers) => {
                    let ptr_dtype = Dtype::pointer(base_dtype);
                    Self::from_ast_pointer_qualifiers(pointer_qualifiers, ptr_dtype)?
                }
                ast::DerivedDeclarator::Array(_array_decl) => todo!(),
                ast::DerivedDeclarator::Function(func_decl) => {
                    let params = func_decl
                        .node
                        .parameters
                        .iter()
                        .map(|p| Self::from_ast_parameter_declaration(&p.node))
                        .collect::<Result<Vec<_>, _>>()?;
                    Self::function(base_dtype, params)
                }
                ast::DerivedDeclarator::KRFunction(kr_func_decl) => {
                    // K&R function is allowed only when it has no parameter
                    assert!(kr_func_decl.is_empty());
                    Self::function(base_dtype, Vec::new())
                }
            };
        }

        let declarator_kind = &declarator.kind;
        match &declarator_kind.node {
            ast::DeclaratorKind::Abstract => panic!("ast::DeclaratorKind::Abstract is unsupported"),
            ast::DeclaratorKind::Identifier(_) => Ok(base_dtype),
            ast::DeclaratorKind::Declarator(declarator) => {
                Self::from_ast_declarator(&declarator.node, base_dtype)
            }
        }
    }

    /// Generate `Dtype` based on parameter declaration
    pub fn from_ast_parameter_declaration(
        parameter_decl: &ast::ParameterDeclaration,
    ) -> Result<Self, DtypeError> {
        let spec = Specifier::from_ast_declaration_specifiers(&parameter_decl.specifiers)?;
        let base_dtype = Self::from_ast_specifiers(spec)?;

        if let Some(declarator) = &parameter_decl.declarator {
            Self::from_ast_declarator(&declarator.node, base_dtype)
        } else {
            Ok(base_dtype)
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

#[derive(Debug, Eq, Clone)]
pub enum RegisterId {
    // `name` field of `Local` is unnecessary, but it is helpful when read printed IR
    Local { name: String, id: usize },
    Arg { id: usize },
    Temp { bid: BlockId, iid: usize },
}

impl RegisterId {
    pub fn local(name: String, id: usize) -> Self {
        Self::Local { name, id }
    }

    pub fn arg(id: usize) -> Self {
        Self::Arg { id }
    }

    pub fn temp(bid: BlockId, iid: usize) -> Self {
        Self::Temp { bid, iid }
    }
}

impl fmt::Display for RegisterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Local { name, id } => write!(f, "%(local:{}:{})", name, id),
            Self::Arg { id } => write!(f, "%(arg:{})", id),
            Self::Temp { bid, iid } => write!(f, "%({}:{})", bid, iid),
        }
    }
}

impl PartialEq<RegisterId> for RegisterId {
    fn eq(&self, other: &RegisterId) -> bool {
        match (self, other) {
            (Self::Local { id, .. }, Self::Local { id: other_id, .. }) => id == other_id,
            (Self::Arg { id }, Self::Arg { id: other_id }) => id == other_id,
            (
                Self::Temp { bid, iid },
                Self::Temp {
                    bid: other_bid,
                    iid: other_iid,
                },
            ) => bid == other_bid && iid == other_iid,
            _ => false,
        }
    }
}

impl Hash for RegisterId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Local { id, .. } => id.hash(state),
            Self::Arg { id } => id.hash(state),
            Self::Temp { bid, iid } => {
                bid.hash(state);
                iid.hash(state);
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Constant {
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
    GlobalVariable {
        name: String,
        dtype: Dtype,
    },
}

impl Constant {
    #[inline]
    pub fn is_integer_constant(&self) -> bool {
        if let Self::Int { .. } = self {
            true
        } else {
            false
        }
    }

    pub fn unit() -> Self {
        Constant::Unit
    }

    #[inline]
    pub fn int(value: u128, dtype: Dtype) -> Self {
        let width = dtype.get_int_width().expect("`dtype` must be `Dtype::Int`");
        let is_signed = dtype.is_signed();

        Constant::Int {
            value,
            width,
            is_signed,
        }
    }

    #[inline]
    pub fn float(value: f64, dtype: Dtype) -> Self {
        let width = dtype
            .get_float_width()
            .expect("`dtype` must be `Dtype::Float`");

        Constant::Float { value, width }
    }

    #[inline]
    pub fn global_variable(name: String, dtype: Dtype) -> Self {
        Self::GlobalVariable { name, dtype }
    }

    pub fn from_ast(constant: &ast::Constant) -> Self {
        match constant {
            ast::Constant::Integer(integer) => {
                let is_signed = !integer.suffix.unsigned;

                let dtype = match integer.suffix.size {
                    ast::IntegerSize::Int => Dtype::INT,
                    ast::IntegerSize::Long => Dtype::LONG,
                    ast::IntegerSize::LongLong => Dtype::LONGLONG,
                }
                .set_signed(is_signed);

                let value = if is_signed {
                    integer.number.parse::<i128>().unwrap() as u128
                } else {
                    integer.number.parse::<u128>().unwrap()
                };

                Self::int(value, dtype)
            }
            ast::Constant::Float(float) => {
                let (dtype, value) = match float.suffix.format {
                    ast::FloatFormat::Float => {
                        // Casting from an f32 to an f64 is perfect and lossless (f32 -> f64)
                        // https://doc.rust-lang.org/stable/reference/expressions/operator-expr.html#type-cast-expressions
                        (Dtype::FLOAT, float.number.parse::<f32>().unwrap() as f64)
                    }
                    ast::FloatFormat::Double => {
                        (Dtype::DOUBLE, float.number.parse::<f64>().unwrap())
                    }
                    ast::FloatFormat::LongDouble => {
                        panic!("`FloatFormat::LongDouble` is_unsupported")
                    }
                    ast::FloatFormat::TS18661Format(_) => {
                        panic!("`FloatFormat::TS18661Format` is_unsupported")
                    }
                };

                Self::float(value, dtype)
            }
            ast::Constant::Character(character) => {
                let dtype = Dtype::CHAR;
                let value = character.parse::<char>().unwrap() as u128;

                Self::int(value, dtype)
            }
        }
    }

    #[inline]
    pub fn from_ast_expression(expr: &ast::Expression) -> Option<Self> {
        if let ast::Expression::Constant(constant) = expr {
            Some(Self::from_ast(&constant.node))
        } else {
            None
        }
    }

    #[inline]
    pub fn from_ast_initializer(initializer: &ast::Initializer) -> Option<Self> {
        if let ast::Initializer::Expression(expr) = &initializer {
            Self::from_ast_expression(&expr.node)
        } else {
            None
        }
    }
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unit => write!(f, "unit"),
            Self::Int { value, .. } => write!(f, "{}", value),
            Self::Float { value, .. } => write!(f, "{}", value),
            Self::GlobalVariable { name, .. } => write!(f, "%{}", name),
        }
    }
}

impl HasDtype for Constant {
    fn dtype(&self) -> Dtype {
        match self {
            Self::Unit => Dtype::unit(),
            Self::Int {
                width, is_signed, ..
            } => Dtype::int(*width).set_signed(*is_signed),
            Self::Float { width, .. } => Dtype::float(*width),
            Self::GlobalVariable { dtype, .. } => Dtype::pointer(dtype.clone()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operand {
    Constant(Constant),
    Register { rid: RegisterId, dtype: Dtype },
}

impl Operand {
    pub fn constant(value: Constant) -> Self {
        Self::Constant(value)
    }

    pub fn register(rid: RegisterId, dtype: Dtype) -> Self {
        Self::Register { rid, dtype }
    }

    pub fn get_register(&self) -> Option<(RegisterId, Dtype)> {
        if let Self::Register { rid, dtype } = self {
            Some((rid.clone(), dtype.clone()))
        } else {
            None
        }
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Constant(value) => write!(f, "{}", value),
            Self::Register { rid, .. } => write!(f, "{}", rid),
        }
    }
}

impl HasDtype for Operand {
    fn dtype(&self) -> Dtype {
        match self {
            Self::Constant(value) => value.dtype(),
            Self::Register { dtype, .. } => dtype.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    // TODO: the variants of Instruction will be added in the future
    BinOp {
        op: ast::BinaryOperator,
        lhs: Operand,
        rhs: Operand,
        dtype: Dtype,
    },
    UnaryOp {
        op: ast::UnaryOperator,
        operand: Operand,
        dtype: Dtype,
    },
    Store {
        ptr: Operand,
        value: Operand,
    },
    Load {
        ptr: Operand,
    },
    Call {
        callee: Operand,
        args: Vec<Operand>,
        return_type: Dtype,
    },
    TypeCast {
        value: Operand,
        target_dtype: Dtype,
    },
}

impl HasDtype for Instruction {
    fn dtype(&self) -> Dtype {
        match self {
            Self::BinOp { dtype, .. } => dtype.clone(),
            Self::UnaryOp { dtype, .. } => dtype.clone(),
            Self::Store { .. } => Dtype::unit(),
            Self::Load { ptr } => ptr
                .dtype()
                .get_pointer_inner()
                .expect("Load instruction must have pointer value as operand")
                .deref()
                .clone()
                .set_const(false),
            Self::Call { return_type, .. } => return_type.clone(),
            Self::TypeCast { target_dtype, .. } => target_dtype.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct BlockId(pub usize);

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "b{}", self.0)
    }
}

// TODO
#[derive(Debug, PartialEq, Clone)]
pub enum BlockExit {
    Jump {
        bid: BlockId,
    },
    ConditionalJump {
        condition: Operand,
        bid_then: BlockId,
        bid_else: BlockId,
    },
    Switch {
        value: Operand,
        default: BlockId,
        cases: Vec<(Constant, BlockId)>,
    },
    Return {
        value: Operand,
    },
    Unreachable,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub instructions: Vec<Instruction>,
    pub exit: BlockExit,
}
