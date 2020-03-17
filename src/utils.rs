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

pub trait Translate<S> {
    type Target;
    type Error;

    fn translate(&mut self, source: &S) -> Result<Self::Target, Self::Error>;
}

pub trait Optimize<T> {
    fn optimize(&mut self, code: &mut T) -> bool;
}

#[derive(Default)]
pub struct Repeat<O> {
    inner: O,
}

impl<T, O1: Optimize<T>, O2: Optimize<T>> Optimize<T> for (O1, O2) {
    fn optimize(&mut self, code: &mut T) -> bool {
        let changed1 = self.0.optimize(code);
        let changed2 = self.1.optimize(code);
        changed1 || changed2
    }
}

impl<T, O: Optimize<T>> Optimize<T> for Repeat<O> {
    fn optimize(&mut self, code: &mut T) -> bool {
        if !self.inner.optimize(code) {
            return false;
        }

        while self.inner.optimize(code) {}
        true
    }
}
