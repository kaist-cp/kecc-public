use itertools::izip;

use core::ops::Deref;

#[macro_export]
/// Ok or executing the given expression.
macro_rules! ok_or {
    ($e:expr, $err:expr) => {{
        match $e {
            Ok(r) => r,
            Err(_) => $err,
        }
    }};
}

#[macro_export]
/// Some or executing the given expression.
macro_rules! some_or {
    ($e:expr, $err:expr) => {{
        match $e {
            Some(r) => r,
            None => $err,
        }
    }};
}

#[macro_export]
/// Ok or exiting the process.
macro_rules! ok_or_exit {
    ($e:expr, $code:expr) => {{
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
/// Ok or exiting the process.
macro_rules! some_or_exit {
    ($e:expr, $code:expr) => {{
        match $e {
            Some(r) => r,
            None => ::std::process::exit($code),
        }
    }};
}

/// TODO(document)
pub trait Translate<S> {
    /// TODO(document)
    type Target;

    /// TODO(document)
    type Error;

    /// TODO(document)
    fn translate(&mut self, source: &S) -> Result<Self::Target, Self::Error>;
}

/// TODO(document)
pub trait AssertSupported {
    /// TODO(document)
    fn assert_supported(&self);
}

/// TODO(document)
pub trait IsEquiv {
    /// TODO(document)
    fn is_equiv(&self, other: &Self) -> bool;
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
