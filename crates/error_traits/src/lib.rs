use std::fmt::{Debug, Display};

#[cfg(feature = "error_stack_dyn_ext")]
use error_stack::{Context, IntoReportCompat, ResultExt};

/// Conversion and adjustment of error reports (after turning usual result to
/// report by calling `.into_report()`).
#[cfg(feature = "error_stack_dyn_ext")]
pub trait ConvReport
    where 
        Self: ResultExt + Sized
{
    fn conv_to<C>(self, convert_to: C) -> error_stack::Result<Self::Ok, C>
        where 
            C: Context
    { self.change_context_lazy(|| convert_to) }

    fn conv_to_and_attach<C, A>(self, convert_to: C, attach: A) -> error_stack::Result<Self::Ok, C>
        where 
            A: Display + Debug + Send + Sync + 'static,
            C: Context
    {
        self.change_context_lazy(|| convert_to)
            .attach_printable_lazy(|| attach)
    }
}

#[cfg(feature = "error_stack_dyn_ext")]
impl<T, C> ConvReport for error_stack::Result<T, C> {}


/// Create report from usual result ( required for `Box(dyn Error)` ).
#[cfg(feature = "error_stack_dyn_ext")]
pub trait IntoReportDyn
{
    type Ok;
    type Err;

    fn into_report_dyn(self) -> error_stack::Result<Self::Ok, Self::Err>;
}

#[cfg(feature = "error_stack_dyn_ext")]
impl<T, E> IntoReportDyn for Result<T, E>
    where 
        E: Send + Sync + Debug + Display + 'static
{
    type Ok = T;
    type Err = anyhow::Error;

    fn into_report_dyn(self) -> error_stack::Result<T, anyhow::Error>
    {
        self.map_err(|dyn_err| anyhow::anyhow!(dyn_err))
            .into_report()
    }
}

type StdResult<T, E> = Result<T, E>;

/// When `Ok()` and `Err` variants of `Result` are the same
/// type, it returns this type whether it's an error, or not.
pub trait MergeOkErr<T>
{
    fn merge_ok_err(self) -> T;
}

impl<T> MergeOkErr<T> for StdResult<T, T>
{
    fn merge_ok_err(self) -> T
    {
        match self
        {
            Ok(ok) => ok,
            Err(err) => err
        }
    }
}

/// This trait provides little helper method that replaces error completely ignoring it.
/// Required in order to get rid of ugly `.map_err(|_| bar)` calls.
pub trait MapErrBy<T, N>
{
    fn map_err_by(self, f: fn() -> N) -> StdResult<T, N>;
}

impl<T, E, N> MapErrBy<T, N> for StdResult<T, E>
{
    fn map_err_by(self, f: fn() -> N) -> StdResult<T, N>
    {
        self.map_err(|_| f())
    }
}

/// If error is present, this trait logs it and returns back.
#[cfg(feature = "log_err")]
pub trait LogErr
{
    fn log_err(self, log_msg: &str) -> Self;
}

impl<T, E> LogErr for StdResult<T, E>
    where
        E: Display
{
    fn log_err(self, log_msg: &str) -> Self
    {
        match self
        {
            Ok(ok) => Ok(ok),
            Err(e) =>
                {
                    log::error!("{log_msg}{e}");
                    Err(e)
                }
        }
    }
}


/// Turns error into a string.
pub trait MapErrToString<T>
{
    fn map_err_to_str(self) -> Result<T, String>;
}

impl<T, E> MapErrToString<T> for StdResult<T, E>
    where E: ToString
{
    fn map_err_to_str(self) -> StdResult<T, String>
    {
        self.map_err(|e| e.to_string())
    }
}


/// Wraps any type into `Ok()` variant of `Result`.
pub trait InOk<T, E>
{
    fn in_ok(self) -> StdResult<T, E>;
}

impl<T, E> InOk<T, E> for T
{
    fn in_ok(self) -> StdResult<T, E>
    {
        Ok(self)
    }
}


/// Wraps any type into `Ok()` variant of `Result`.
pub trait InErr<T, E>
{
    fn in_err(self) -> StdResult<T, E>;
}

impl<T, E> InErr<T, E> for E
{
    fn in_err(self) -> StdResult<T, E>
    {
        Err(self)
    }
}


