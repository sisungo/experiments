//! The data verification framework.

use std::fmt::Display;

/// Defines a verification standard.
#[macro_export]
macro_rules! def_verify {
    ($vis:vis $name:ident<$t:ty> = $f:expr) => {
        #[derive(Debug, Default, Clone, Copy)]
        $vis struct $name;
        impl $crate::verify::Verify<$t> for $name {
            type Err = $crate::verify::DefaultVerifyError;

            fn verify(self, value: &$t) -> Result<(), Self::Err> {
                if $f(value) {
                    Ok(())
                } else {
                    Err(::std::default::Default::default())
                }
            }
        }
    };
    ($vis:vis $name:ident<$t:ty>(err: $e:ty = $ed:expr) = $f:expr) => {
        #[derive(Debug, Default, Clone, Copy)]
        $vis struct $name;
        impl $crate::verify::Verify<$t> for $name {
            type Err = $e;

            fn verify(self, value: &$t) -> Result<(), Self::Err> {
                if $f(value) {
                    Ok(())
                } else {
                    Err($ed)
                }
            }
        }
    };
    ($vis:vis $name:ident<$t:ty, $e:ty> = $f:expr) => {
        #[derive(Debug, Default, Clone, Copy)]
        $vis struct $name;
        impl $crate::verify::Verify<$t> for $name {
            type Err = $e;

            fn verify(self, value: &$t) -> Result<(), Self::Err> {
                $f(value)
            }
        }
    };
}

/// A verification standard.
pub trait Verify<T: ?Sized> {
    type Err;

    /// Verifies the data. Returns `true` if the data is verified.
    fn verify(self, value: &T) -> Result<(), Self::Err>;
}

impl<F, T> Verify<T> for F
where
    F: FnOnce(&T) -> bool,
{
    type Err = DefaultVerifyError;

    fn verify(self, value: &T) -> Result<(), DefaultVerifyError> {
        if self(value) {
            Ok(())
        } else {
            Err(DefaultVerifyError)
        }
    }
}

pub trait VerifyExt {
    /// Verifies the data by given standard.
    fn verify_by<S: Verify<Self> + Default>(&self) -> Result<(), S::Err>;

    /// Returns `true` if the data is valid in given standard.
    fn is_a_valid<S: Verify<Self> + Default>(&self) -> bool {
        self.verify_by::<S>().is_ok()
    }
}
impl VerifyExt for str {
    fn verify_by<S: Verify<Self> + Default>(&self) -> Result<(), S::Err> {
        S::default().verify(self)
    }
}

/// The default verification error.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultVerifyError;
impl Display for DefaultVerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "verification failed")
    }
}
impl std::error::Error for DefaultVerifyError {}

/// Verifies data using given standard optionally, passes if the value is [`None`].
pub fn verify_option<S: Verify<T> + Default, T: ?Sized>(value: Option<&T>) -> Result<(), S::Err> {
    match value {
        Some(x) => S::default().verify(x),
        None => Ok(()),
    }
}
