#[macro_export]
/// Ok or exiting the process.
macro_rules! ok_or_exit {
    ($e:expr_2021, $code:expr_2021) => {{
        match $e {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{:?}", e);
                ::std::process::exit($code);
            }
        }
    }};
}

#[macro_export]
/// Some or exiting the process.
macro_rules! some_or_exit {
    ($e:expr_2021, $code:expr_2021) => {{
        match $e {
            Some(r) => r,
            None => ::std::process::exit($code),
        }
    }};
}

/// Translates `S` to [`Translate::Target`].
// TODO: Should this be in utils?
pub trait Translate<S> {
    /// The type to translate to.
    type Target;

    /// The error type.
    type Error;

    /// Translate `source` to `Self::Target`.
    fn translate(&mut self, source: &S) -> Result<Self::Target, Self::Error>;
}

/// Trait to check if a type can be translated.
pub trait AssertSupported {
    /// Assert that the type can be translated.
    ///
    /// # Panics
    ///
    /// Panics if the type can't be translated.
    // TODO: should return a boolean.
    fn assert_supported(&self);
}

/// Essentially the same as [`PartialEq`].
///
/// Exists to check equaility on some foreign types.
pub trait IsEquiv {
    /// See [`PartialEq::eq`].
    fn is_equiv(&self, other: &Self) -> bool;
}

impl<T: IsEquiv> IsEquiv for Box<T> {
    fn is_equiv(&self, other: &Self) -> bool {
        (**self).is_equiv(&**other)
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
        self.len() == other.len() && self.iter().zip(other).all(|(lhs, rhs)| lhs.is_equiv(rhs))
    }
}
