//! Helper module providing some macros to print errors or unwrap values.
//!
//! This module exists because I could not find an error-handling crate that
//! let me decide how to deal with errors in the following way:
//!
//! - Continue execution.
//! - Exit with a code.
//!
//! While priting an error message in both cases.

/// Unwraps the result in the first argument, printing the error and continuing
/// the current loop if it is an `Err`.
///
/// Will add a `-- error: {error}` at the end of the given error text.
#[macro_export]
macro_rules! continue_error {
    ($value:expr, $($arg:tt)*) => {
        $crate::__error!(continue, $value, $($arg)*);
    };
}

/// Unwraps the result in the first argument, printing the error and returning
/// if it is an `Err`.
///
/// Will add a `-- error: {error}` at the end of the given error text.
#[macro_export]
macro_rules! return_error {
    ($value:expr, $($arg:tt)*) => {
        $crate::__error!(return, $value, $($arg)*);
    };
}

/// Unwraps the result in the second argument, printing the error and exiting
/// the current process with the given error code if it is an `Err`.
///
/// Will add a `-- error: {error}` at the end of the given error text.
#[macro_export]
macro_rules! code_error {
    ($code: expr, $value:expr, $($arg:tt)*) => {
        $crate::__error!(::std::process::exit($code), $value, $($arg)*);
    };
}

/// Internl helper macro to avoid repeating pretty much the same code several
/// times.
///
/// Will add a `-- error: {error}` at the end of the given error text.
#[macro_export]
macro_rules! __error {
    ($out:expr, $value:expr, $($arg:tt)*) => {
        match $value {
            ::std::result::Result::Ok(v) => v,
            ::std::result::Result::Err(e) => {
                ::std::eprint!($($arg)*);
                ::std::eprintln!(" -- error: {}", e);
                $out;
            },
        }
    };
}

#[cfg(test)]
type TestResult = Result<usize, usize>;

#[test]
fn continue_error() {
    let mut res = 0;
    // Should exit on the second iteration else it will never break.
    loop {
        if res != 0 {
            break;
        }
        res = continue_error!(TestResult::Ok(4), "");
        assert_eq!(res, 4);
    }

    // Should always continue, never reaching the false.
    for i in 0..10 {
        continue_error!(TestResult::Err(i), "");
        assert!(false, "Should never be reached");
    }
}

#[test]
fn return_error() {
    fn early_return() {
        return_error!(TestResult::Err(4), "");
        assert!(false, "Should never be reached");
    }

    early_return();

    let mut returned_early = true;
    let mut not_early_return = || {
        return_error!(TestResult::Ok(2), "");
        returned_early = false;
    };
    not_early_return();

    assert!(!returned_early, "Value should be false");
}
