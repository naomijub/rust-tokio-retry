use std::error;
use std::fmt;

use std::time::Duration;

/// `Error` is the error value in an actions's retry result.
///
/// Based on the two possible values, the operation
/// may be retried.
pub enum Error<E> {
    /// `Permanent` means that it's impossible to execute the operation
    /// successfully. This error is an early return from the retry operation.
    Permanent(E),

    /// `Transient` means that the error is temporary. If the `retry_after` is `None`
    /// the operation should be retried according to the defined strategy policy, else after
    /// the specified duration. Useful for handling ratelimits like a HTTP 429 response.
    Transient {
        err: E,
        retry_after: Option<Duration>,
    },
}

impl<E> Error<E> {
    // Creates an permanent error.
    pub fn permanent(err: E) -> Self {
        Error::Permanent(err)
    }

    // Creates a Result::Err container with an permanent error.
    pub fn to_permanent<T>(err: E) -> Result<T, Self> {
        Err(Error::Permanent(err))
    }

    // Creates an transient error which is retried according to the defined strategy
    // policy.
    pub fn transient(err: E) -> Self {
        Error::Transient {
            err,
            retry_after: None,
        }
    }

    // Creates a Result::Err container with an transient error which
    // is retried according to the defined strategy policy.
    pub fn to_transient<T>(err: E) -> Result<T, Self> {
        Err(Error::Transient {
            err,
            retry_after: None,
        })
    }

    /// Creates a Result::Err container with a transient error which
    /// is retried after the specified duration.
    /// Useful for handling ratelimits like a HTTP 429 response.
    pub fn to_retry_after<T>(err: E, duration: Duration) -> Result<T, Self> {
        Err(Error::Transient {
            err,
            retry_after: Some(duration),
        })
    }

    /// Creates a transient error which is retried after the specified duration.
    /// Useful for handling ratelimits like a HTTP 429 response.
    pub fn retry_after(err: E, duration: Duration) -> Self {
        Error::Transient {
            err,
            retry_after: Some(duration),
        }
    }
}

impl<E> fmt::Display for Error<E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Error::Permanent(ref err)
            | Error::Transient {
                ref err,
                retry_after: _,
            } => err.fmt(f),
        }
    }
}

impl<E> fmt::Debug for Error<E>
where
    E: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let (name, err) = match *self {
            Error::Permanent(ref err) => ("Permanent", err as &dyn fmt::Debug),
            Error::Transient {
                ref err,
                retry_after: _,
            } => ("Transient", err as &dyn fmt::Debug),
        };
        f.debug_tuple(name).field(err).finish()
    }
}

impl<E> error::Error for Error<E>
where
    E: error::Error,
{
    fn description(&self) -> &str {
        match *self {
            Error::Permanent(_) => "permanent error",
            Error::Transient { .. } => "transient error",
        }
    }

    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::Permanent(ref err)
            | Error::Transient {
                ref err,
                retry_after: _,
            } => err.source(),
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        self.source()
    }
}

/// By default all errors are transient. Permanent errors can
/// be constructed explicitly. This implementation is for making
/// the question mark operator (?) and the `try!` macro to work.
impl<E> From<E> for Error<E> {
    fn from(err: E) -> Error<E> {
        Error::Transient {
            err,
            retry_after: None,
        }
    }
}

impl<E> PartialEq for Error<E>
where
    E: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Error::Permanent(ref self_err), Error::Permanent(ref other_err)) => {
                self_err == other_err
            }
            (
                Error::Transient {
                    err: self_err,
                    retry_after: self_retry_after,
                },
                Error::Transient {
                    err: other_err,
                    retry_after: other_retry_after,
                },
            ) => self_err == other_err && self_retry_after == other_retry_after,
            _ => false,
        }
    }
}

pub trait MapErr<T, E> {
    fn map_transient(self) -> Result<T, Error<E>>;
    fn map_permanent(self) -> Result<T, Error<E>>;
}

impl<T, E> MapErr<T, E> for Result<T, E> {
    #[inline]
    fn map_transient(self) -> Result<T, Error<E>> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(Error::transient(e)),
        }
    }

    #[inline]
    fn map_permanent(self) -> Result<T, Error<E>> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(Error::permanent(e)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::error::Error as StdError;
    use std::fmt;

    #[test]
    fn create_permanent_error() {
        let e = Error::permanent("err");
        assert_eq!(e, Error::Permanent("err"));
    }

    #[test]
    fn create_transient_error() {
        let e = Error::transient("err");
        assert_eq!(
            e,
            Error::Transient {
                err: "err",
                retry_after: None
            }
        );
    }

    #[test]
    fn create_transient_error_with_retry_after() {
        let retry_after = Duration::from_secs(42);
        let e = Error::retry_after("err", retry_after);
        assert_eq!(
            e,
            Error::Transient {
                err: "err",
                retry_after: Some(retry_after),
            }
        );
    }

    #[test]
    fn map_transient_keeps_ok() {
        let result: Result<i32, Error<()>> = Ok(42).map_transient();
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn map_transient_maps_err() {
        let result: Result<(), Error<&str>> = Err("err").map_transient();
        assert_eq!(
            result,
            Err::<(), Error<&str>>(Error::Transient {
                err: "err",
                retry_after: None
            })
        );
    }

    #[test]
    fn map_permanent_keeps_ok() {
        let result: Result<i32, Error<()>> = Ok(42).map_permanent();
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn map_permanent_maps_err() {
        let result: Result<(), Error<&str>> = Err("err").map_permanent();
        assert_eq!(result, Err(Error::Permanent("err")));
    }

    #[test]
    fn fmt_permanent_error() {
        let error = Error::Permanent("permanent error");
        let formatted = format!("{}", error);
        assert_eq!(formatted, "permanent error");
    }

    #[test]
    fn fmt_transient_error() {
        let error = Error::Transient {
            err: "transient error",
            retry_after: None,
        };
        let formatted = format!("{}", error);
        assert_eq!(formatted, "transient error");
    }

    #[test]
    fn debug_permanent_error() {
        let error = Error::Permanent("permanent error");
        let debug = format!("{:?}", error);
        assert_eq!(debug, "Permanent(\"permanent error\")");
    }

    #[test]
    fn debug_transient_error() {
        let error = Error::Transient {
            err: "transient error",
            retry_after: None,
        };
        let debug = format!("{:?}", error);
        assert_eq!(debug, "Transient(\"transient error\")");
    }

    #[test]
    fn description_permanent_error() {
        let error = Error::permanent(MyError("permanent error"));
        assert_eq!(error.description(), "permanent error");
    }

    #[test]
    fn description_transient_error() {
        let error = Error::transient(MyError("transient error"));
        assert_eq!(error.description(), "transient error");
    }

    #[test]
    fn source_permanent_error() {
        let error: Result<(), Error<MyError>> = Error::to_retry_after(
            MyError("transient error"),
            std::time::Duration::from_secs(1),
        );
        assert!(error.unwrap_err().source().is_none());
    }

    #[test]
    fn source_transient_error() {
        let error = Error::retry_after(
            MyError("transient error"),
            std::time::Duration::from_secs(1),
        );
        assert!(error.source().is_none());
    }

    #[test]
    fn cause_permanent_error() {
        let error = Error::permanent(MyError("permanent error"));
        assert!(error.cause().is_none());
    }

    #[test]
    fn cause_transient_error() {
        let error = Error::transient(MyError("transient error"));
        assert!(error.cause().is_none());
    }

    #[derive(Debug)]
    pub struct MyError(pub &'static str);
    impl fmt::Display for MyError {
        fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            write!(f, "{}", self.0)
        }
    }
    impl StdError for MyError {
        fn description(&self) -> &str {
            self.0
        }

        fn source(&self) -> Option<&(dyn StdError + 'static)> {
            None
        }

        fn cause(&self) -> Option<&dyn StdError> {
            self.source()
        }
    }
}
