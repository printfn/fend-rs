#![forbid(unsafe_code)]
#![forbid(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::use_self)]
#![forbid(clippy::needless_borrow)]
#![forbid(clippy::cognitive_complexity)]
#![forbid(unreachable_pub)]
#![doc(html_root_url = "https://docs.rs/fend-core/0.1.14")]

mod ast;
mod datetime;
mod error;
mod eval;
mod interrupt;
mod lexer;
mod num;
mod parser;
mod scope;
mod units;
mod value;

pub use interrupt::Interrupt;

/// This contains the result of a computation.
#[derive(PartialEq, Eq, Debug)]
pub struct FendResult {
    main_result: String,
    other_info: Vec<String>,
}

impl FendResult {
    /// This retrieves the main result of the computation.
    #[must_use]
    pub fn get_main_result(&self) -> &str {
        self.main_result.as_str()
    }

    /// This retrieves a list of other results of the computation. It is less
    /// stable than the main result, and should only be shown for when used
    /// interactively.
    pub fn get_other_info(&self) -> impl Iterator<Item = &str> {
        self.other_info.iter().map(String::as_str)
    }
}

/// This struct contains context used for `fend`. It should only be created once
/// at startup.
#[derive(Clone)]
pub struct Context {
    elapsed_unix_time_ms: Option<u64>,
    timezone_offset_secs: Option<i64>,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    /// Create a new context instance. This can be fairly slow, and should
    /// only be done once if possible.
    #[must_use]
    pub fn new() -> Self {
        Self {
            elapsed_unix_time_ms: None,
            timezone_offset_secs: None,
        }
    }

    /// Override the current time. This must be the number of elapsed milliseconds
    /// since January 1, 1970 at midnight UTC, ignoring leap seconds in the same way
    /// as unix time.
    pub fn override_current_unix_time_ms(&mut self, ms_since_1970: u64) {
        self.elapsed_unix_time_ms = Some(ms_since_1970);
    }

    /// Override the current time zone offset to UTC, in seconds.
    pub fn override_timezone_offset(&mut self, tz_offset_secs: i64) {
        self.timezone_offset_secs = Some(tz_offset_secs);
    }
}

/// This function evaluates a string using the given context. Any evaluation using this
/// function cannot be interrupted.
///
/// For example, passing in the string `"1 + 1"` will return a result of `"2"`.
///
/// # Errors
/// It returns an error if the given string is invalid.
/// This may be due to parser or runtime errors.
pub fn evaluate(input: &str, context: &mut Context) -> Result<FendResult, String> {
    evaluate_with_interrupt(input, context, &interrupt::Never::default())
}

/// This function evaluates a string using the given context and the provided
/// Interrupt object.
///
/// For example, passing in the string `"1 + 1"` will return a result of `"2"`.
///
/// # Errors
/// It returns an error if the given string is invalid.
/// This may be due to parser or runtime errors.
pub fn evaluate_with_interrupt(
    input: &str,
    context: &mut Context,
    int: &impl Interrupt,
) -> Result<FendResult, String> {
    if input.is_empty() {
        // no or blank input: return no output
        return Ok(FendResult {
            main_result: "".to_string(),
            other_info: vec![],
        });
    }
    let result = match eval::evaluate_to_string(input, None, context, int) {
        Ok(value) => value,
        // TODO: handle different interrupt values
        Err(error::IntErr::Interrupt(_)) => return Err("Interrupted".to_string()),
        Err(error::IntErr::Error(e)) => return Err(e),
    };
    Ok(FendResult {
        main_result: result,
        other_info: vec![],
    })
}

const fn get_version_as_str() -> &'static str {
    "0.1.14"
}

/// Returns the current version of `fend-core`.
#[must_use]
pub fn get_version() -> String {
    get_version_as_str().to_string()
}

/// Deprecated: use `get_version()` instead.
#[must_use]
#[deprecated = "Use `get_version()` instead"]
pub fn get_extended_version() -> String {
    get_version()
}
