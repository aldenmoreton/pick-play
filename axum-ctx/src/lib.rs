//! Axum error handling inspired by [`anyhow`](https://docs.rs/anyhow)
//!
//! ## Comparison to [`anyhow`](https://docs.rs/anyhow)
//!
//! Assume a function `can_fail` that returns `Result<T, E>` or `Option<T>`.
//!
//! With `anyhow`, you can do the following:
//!
//! ```rust
//! use anyhow::{Context, Result};
//!
//! # fn can_fail() -> Option<()> {
//! #     None
//! # }
//! #
//! # fn example() -> Result<()> {
//! let value = can_fail().context("Error message")?;
//! # Ok(())
//! # }
//! ```
//!
//! For many types of programs, this is more than enough.
//! But for web backends, you don't only want to report an error.
//! You want to return a response with a proper HTTP status code.
//! Then you want to log the error (using [`tracing`]).
//! This is what `axum-ctx` does:
//!
//! ```rust
//! // Use a wildcard for the best user experience
//! use axum_ctx::*;
//!
//! # fn can_fail() -> Option<()> {
//! #     None
//! # }
//! #
//! # fn example() -> RespResult<()> {
//! let value = can_fail().ctx(StatusCode::BAD_REQUEST).log_msg("Error message")?;
//! # Ok(())
//! # }
//! ```
//!
//! If an error occurs, the user gets the error message "400 Bad Request" corresponding to the status code that you specified.
//! But you can replace this default message with a custom error message to be shown to the user:
//!
//! ```rust
//! # use axum_ctx::*;
//! #
//! # fn can_fail() -> Option<()> {
//! #     None
//! # }
//! #
//! # fn example() -> RespResult<()> {
//! let value = can_fail()
//!     .ctx(StatusCode::UNAUTHORIZED)
//!     // Shown to the user
//!     .user_msg("You are not allowed to access this resource!")
//!     // NOT shown to the user, only for the log
//!     .log_msg("Someone tries to pentest you")?;
//! # Ok(())
//! # }
//! ```
//!
//! A second call of `user_msg` replaces the the user error message.
//! But calling `log_msg` multiple times creates a backtrace:
//!
//! ```rust
//! # use axum_ctx::*;
//! #
//! # fn can_fail() -> Option<()> {
//! #     None
//! # }
//! #
//! # fn example() -> RespResult<()> {
//! fn returns_resp_result() -> RespResult<()> {
//!     can_fail().ctx(StatusCode::NOT_FOUND).log_msg("Inner error message")
//! }
//!
//! let value = returns_resp_result()
//!     .log_msg("Outer error message")?;
//! # Ok(())
//! # }
//! ```
//!
//! The code above leads to the following log message:
//!
//! ```text
//! 2024-05-08T22:17:53.769240Z  INFO axum_ctx: 404 Not Found
//!   0: Outer error message
//!   1: Inner error message
//! ```
//!
//! ## Lazy evaluation
//! Similar to [`with_context`](https://docs.rs/anyhow/1.0.83/anyhow/trait.Context.html#tymethod.with_context) provided by `anyhow`, `axum-ctx` also supports lazy evaluation of [messages](Message).
//! You just provide a closure to `user_msg` or `log_msg`:
//!
//! ```rust
//! # use axum_ctx::*;
//! #
//! # fn can_fail() -> Option<()> {
//! #     None
//! # }
//! #
//! # fn example() -> RespResult<()> {
//! let resource_name = "foo";
//! let value = can_fail()
//!     .ctx(StatusCode::UNAUTHORIZED)
//!     .user_msg(|| format!("You are not allowed to access the resource {resource_name}!"))
//!     .log_msg(|| format!("Someone tries to access {resource_name}"))?;
//! # Ok(())
//! # }
//! ```
//!
//! `.user_msg(format!("…"))` creates the string on the heap even if `can_fail` didn't return `Err` (or `None` for options).
//! `.user_msg(|| format!("…"))` (a closure with two pipes `||`) only creates the string if `Err`/`None` actually occurred.
//!
//! ## Logging
//!
//! `axum-ctx` uses [`tracing`] for logging.
//! This means that you need to [initialize a tracing subscriber](https://docs.rs/tracing-subscriber/0.3.18/tracing_subscriber/fmt/index.html) in your program first before being able to see the log messages of `axum-ctx`.
//!
//! `axum-ctx` automatically chooses a [tracing level](tracing::Level) depending on the chosen status code.
//! Here is the default range mapping (status codes less than 100 or bigger than 999 are not allowed):
//!
//! | Status Code  | Level   |
//! | ------------ | ------- |
//! | `100..400`   | `Debug` |
//! | `400..500`   | `Info`  |
//! | `500..600`   | `Error` |
//! | `600..1000`  | `Trace` |
//!
//! You can change the default level for one or more status codes using [`change_tracing_level`] on program initialization
//!
//! ## Example
//!
//! Assume that you want to get all salaries from a database and then return their maximum from an Axum API.
//!
//! The steps required:
//!
//! > **1.** Get all salaries from the database. This might fail for example if the database isn't reachable
//! >
//! > ➡️ You need to handle a `Result`
//! >
//! > **2.** Determine the maximum salary. But if there were no salaries in the database, there is no maximum
//! >
//! > ➡️ You need to handle an `Option`
//! >
//! > **3.** Return the maximum salary as JSON.
//!
//! First, let's define a function to get all salaries:
//!
//! ```rust
//! async fn salaries_from_db() -> Result<Vec<f64>, String> {
//!     // Imagine getting this error while trying to connect to the database.
//!     Err(String::from("Database unreachable"))
//! }
//! ```
//!
//! Now, let's see how to do proper handling of `Result` and `Option` in an Axum handler:
//!
//! ```rust
//! use axum::Json;
//! use http::StatusCode;
//! use tracing::{error, info};
//!
//! # async fn salaries_from_db() -> Result<Vec<f64>, String> {
//! #     // Imagine getting this error while trying to connect to the database.
//! #     Err(String::from("Database unreachable"))
//! # }
//! #
//! async fn max_salary() -> Result<Json<f64>, (StatusCode, &'static str)> {
//!     let salaries = match salaries_from_db().await {
//!         Ok(salaries) => salaries,
//!         Err(error) => {
//!             error!("Failed to get all salaries from the DB\n{error}");
//!             return Err((
//!                 StatusCode::INTERNAL_SERVER_ERROR,
//!                 "Something went wrong. Please try again later",
//!             ));
//!         }
//!     };
//!
//!     match salaries.iter().copied().reduce(f64::max) {
//!         Some(max_salary) => Ok(Json(max_salary)),
//!         None => {
//!             info!("The maximum salary was requested although there are no salaries");
//!             Err((StatusCode::NOT_FOUND, "There are no salaries yet!"))
//!         }
//!     }
//! }
//! ```
//!
//! Now, compare the code above with the one below that uses `axum-ctx`:
//!
//! ```rust
//! # use axum::Json;
//! use axum_ctx::*;
//! # use tracing::{error, info};
//!
//! # async fn salaries_from_db() -> Result<Vec<f64>, String> {
//! #     // Imagine getting this error while trying to connect to the database.
//! #     Err(String::from("Database unreachable"))
//! # }
//! #
//! async fn max_salary() -> RespResult<Json<f64>> {
//!     salaries_from_db()
//!         .await
//!         .ctx(StatusCode::INTERNAL_SERVER_ERROR)
//!         .user_msg("Something went wrong. Please try again later")
//!         .log_msg("Failed to get all salaries from the DB")?
//!         .iter()
//!         .copied()
//!         .reduce(f64::max)
//!         .ctx(StatusCode::NOT_FOUND)
//!         .user_msg("There are no salaries yet!")
//!         .log_msg("The maximum salary was requested although there are no salaries")
//!         .map(Json)
//! }
//! ```
//!
//! Isn't that a wonderful chain? ⛓️ It is basically a "one-liner" if you ignore the pretty formatting.
//!
//! The user gets the message "Something went wrong. Please try again later". In your terminal, you get the following log message:
//!
//! ```text
//! 2024-05-08T22:17:53.769240Z  ERROR axum_ctx: Something went wrong. Please try again later
//!   0: Failed to get all salaries from the DB
//!   1: Database unreachable
//! ```
//!
//! "What about `map_or_else` and `ok_or_else`?", you might ask.
//! You can use them if you prefer chaining like me, but the code will not be as concise as the one above with `axum_ctx`.
//! You can compare:
//!
//! ```rust
//! # use axum::Json;
//! # use http::StatusCode;
//! # use tracing::{error, info};
//! #
//! # async fn salaries_from_db() -> Result<Vec<f64>, String> {
//! #     // Imagine getting this error while trying to connect to the database.
//! #     Err(String::from("Database unreachable"))
//! # }
//! #
//! async fn max_salary() -> Result<Json<f64>, (StatusCode, &'static str)> {
//!     salaries_from_db()
//!         .await
//!         .map_err(|error| {
//!             error!("Failed to get all salaries from the DB\n{error}");
//!             (
//!                 StatusCode::INTERNAL_SERVER_ERROR,
//!                 "Something went wrong. Please try again later",
//!             )
//!         })?
//!         .iter()
//!         .copied()
//!         .reduce(f64::max)
//!         .ok_or_else(|| {
//!             info!("The maximum salary was requested although there are no salaries");
//!             (StatusCode::NOT_FOUND, "There are no salaries yet!")
//!         })
//!         .map(Json)
//! }
//! ```

use axum_core::response::{IntoResponse, Response};
use std::{borrow::Cow, fmt};
use tracing::{event, Level};

pub use http::StatusCode;

static mut STATUS_CODE_TRACE_LEVEL: [TracingLevel; 1000] = {
    let mut array = [TracingLevel::Trace; 1000];

    let mut ind = 100;
    while ind < 400 {
        array[ind] = TracingLevel::Debug;
        ind += 1;
    }
    while ind < 500 {
        array[ind] = TracingLevel::Info;
        ind += 1;
    }
    while ind < 600 {
        array[ind] = TracingLevel::Error;
        ind += 1;
    }

    array
};

/// [`Result`] with [`RespErr`] as the error variant.
pub type RespResult<T> = Result<T, RespErr>;

/// The tracing level that maps to [`tracing::Level`].
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TracingLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Change the default tracing level for a status code.
///
/// Should only be used on program initialization.
///
/// # Panics
/// Panics if the status code is less than 100 or greater than 999.
///
/// # Examples
/// ```
/// # use axum_ctx::{change_tracing_level, TracingLevel};
/// for status_code in 100..200 {
///     change_tracing_level(status_code, TracingLevel::Info);
/// }
/// ```
///
/// Examples of panics:
///
/// ```should_panic
/// # use axum_ctx::{change_tracing_level, TracingLevel};
/// change_tracing_level(99, TracingLevel::Info); // Less than 100
/// ```
///
/// ```should_panic
/// # use axum_ctx::{change_tracing_level, TracingLevel};
/// change_tracing_level(1000, TracingLevel::Info); // Greater than 999
/// ```
pub fn change_tracing_level(status_code: usize, level: TracingLevel) {
    assert!(
        (100..1000).contains(&status_code),
        "The status code has to be >=100 and <1000",
    );

    unsafe { STATUS_CODE_TRACE_LEVEL[status_code] = level };
}

/// An error message.
#[derive(Debug)]
pub struct Message(pub Cow<'static, str>);

impl From<&'static str> for Message {
    fn from(value: &'static str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for Message {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<F, V> From<F> for Message
where
    F: FnOnce() -> V,
    V: Into<Self>,
{
    fn from(f: F) -> Self {
        f().into()
    }
}

#[derive(Debug)]
enum ResponseKind {
    /// Shows a default message to the user.
    DefaultMessage,
    /// Shows a custom message to the user.
    CustomMessage(Message),
    /// A custom response.
    Response(Response),
}

/// An error to be used as the error variant of a request handler.
///
/// Often initialized by using [`RespErrCtx::ctx`] on [`Result`], [`Option`] or [`Response`].
/// But it can also be initialized by [`RespErr::new`].
///
/// # Examples
///
/// ```
/// use axum_ctx::*;
///
/// fn can_fail() -> Result<(), std::io::Error> {
///     // …
///     # Ok(())
/// }
///
/// async fn get() -> Result<StatusCode, RespErr> {
///     can_fail()
///         .ctx(StatusCode::INTERNAL_SERVER_ERROR)
///         .user_msg("Sorry for disappointing you. Do you want a cookie?")
///         .log_msg("Failed to do …. Blame Max Mustermann")?;
///     // …
///     Ok(StatusCode::OK)
/// }
/// ```
///
/// Using the type alias [`RespResult`] is recommended:
///
/// ```
/// # use axum_ctx::*;
/// async fn get() -> RespResult<StatusCode> {
///     // …
///     # Ok(StatusCode::OK)
/// }
/// ```
#[derive(Debug)]
pub struct RespErr {
    pub status_code: StatusCode,
    log_messages: Vec<Message>,
    response_kind: ResponseKind,
}

impl RespErr {
    /// Initialize with a status.
    #[must_use]
    pub const fn new(status_code: StatusCode) -> Self {
        Self {
            status_code,
            log_messages: Vec::new(),
            response_kind: ResponseKind::DefaultMessage,
        }
    }

    /// Optionally add a custom user error message.
    #[must_use]
    pub fn user_msg(mut self, message: impl Into<Message>) -> Self {
        self.response_kind = ResponseKind::CustomMessage(message.into());

        self
    }

    /// Optionally add an error message to be showed in the log.
    /// It will not be shown to the user!
    #[must_use]
    pub fn log_msg(mut self, error: impl Into<Message>) -> Self {
        self.log_messages.push(error.into());

        self
    }
}

impl fmt::Display for RespErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.response_kind {
            ResponseKind::DefaultMessage => self.status_code.fmt(f)?,
            ResponseKind::CustomMessage(message) => f.write_str(&message.0)?,
            ResponseKind::Response(..) => (),
        }

        for (ind, e) in self.log_messages.iter().rev().enumerate() {
            f.write_fmt(format_args!("\n  {ind}: {}", e.0))?;
        }

        Ok(())
    }
}

impl IntoResponse for RespErr {
    /// Log the error, set the HTTP status code and return the response.
    fn into_response(self) -> Response {
        let ind = self.status_code.as_u16() as usize;

        match unsafe { std::ptr::addr_of!(STATUS_CODE_TRACE_LEVEL).as_ref().unwrap().get(ind) } {
            Some(TracingLevel::Trace) => event!(Level::TRACE, "{self}"),
            Some(TracingLevel::Debug) => event!(Level::DEBUG, "{self}"),
            Some(TracingLevel::Info) => event!(Level::INFO, "{self}"),
            Some(TracingLevel::Warn) => event!(Level::WARN, "{self}"),
            Some(TracingLevel::Error) => event!(Level::ERROR, "{self}"),
            None => (),
        }

        let mut response = match self.response_kind {
            ResponseKind::DefaultMessage => self.status_code.to_string().into_response(),
            ResponseKind::CustomMessage(message) => message.0.into_response(),
            ResponseKind::Response(r) => r,
        };

        *response.status_mut() = self.status_code;

        response
    }
}

/// Conversion to a `Result` with [`RespErr`] as the error.
///
/// Inspired by `anyhow::Context`, especially the conversion from [`Result<T, E>`](Result) or [`Option<T>`](Option) to `Result<T, RespErr>`.
///
/// After this conversion, you can add a user and/or error message using [`RespErrExt`].
pub trait RespErrCtx<T> {
    /// Convert by adding a status as a context.
    fn ctx(self, status_code: StatusCode) -> Result<T, RespErr>;
}

impl<T, E> RespErrCtx<T> for Result<T, E>
where
    E: fmt::Display,
{
    /// The error is used as a log error message.
    fn ctx(self, status_code: StatusCode) -> Result<T, RespErr> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(RespErr::new(status_code).log_msg(e.to_string())),
        }
    }
}

impl<T> RespErrCtx<T> for Option<T> {
    #[inline]
    fn ctx(self, status_code: StatusCode) -> Result<T, RespErr> {
        match self {
            Some(v) => Ok(v),
            None => Err(RespErr::new(status_code)),
        }
    }
}

impl<T> RespErrCtx<T> for Response {
    fn ctx(self, status_code: StatusCode) -> Result<T, RespErr> {
        Err(RespErr {
            status_code,
            log_messages: Vec::new(),
            response_kind: ResponseKind::Response(self),
        })
    }
}

/// Addition of custom user and log error messages to a `Result<T, RespErr>`.
pub trait RespErrExt<T> {
    /// Add a custom user error message.
    ///
    /// See [`RespErr::user_msg`](crate::RespErr::user_msg).
    fn user_msg(self, message: impl Into<Message>) -> Result<T, RespErr>;

    /// Add a log error message.
    ///
    /// See [`RespErr::log_msg`](crate::RespErr::log_msg).
    fn log_msg(self, error: impl Into<Message>) -> Result<T, RespErr>;
}

impl<T> RespErrExt<T> for Result<T, RespErr> {
    #[inline]
    fn user_msg(self, message: impl Into<Message>) -> Self {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(e.user_msg(message)),
        }
    }

    #[inline]
    fn log_msg(self, error: impl Into<Message>) -> Self {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(e.log_msg(error)),
        }
    }
}
