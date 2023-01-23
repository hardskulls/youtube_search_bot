
#[cfg(feature = "error_stack_dyn_ext")]
mod err_stack_ext;
#[cfg(feature = "log_err")]
mod log_err;

#[cfg(feature = "error_stack_dyn_ext")]
pub use err_stack_ext::*;
#[cfg(feature = "log_err")]
pub use log_err::*;

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
    fn map_err_by(self, f: impl Fn() -> N) -> StdResult<T, N>;
}

impl<T, E, N> MapErrBy<T, N> for StdResult<T, E>
{
    fn map_err_by(self, f: impl Fn() -> N) -> StdResult<T, N>
    {
        self.map_err(|_| f())
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
pub trait WrapInOk<T, E>
{
    fn in_ok(self) -> StdResult<T, E>;
}

impl<T, E> WrapInOk<T, E> for T
{
    fn in_ok(self) -> StdResult<T, E>
    {
        Ok(self)
    }
}


/// Wraps any type into `Ok()` variant of `Result`.
pub trait WrapInErr<T, E>
{
    fn in_err(self) -> StdResult<T, E>;
}

impl<T, E> WrapInErr<T, E> for E
{
    fn in_err(self) -> StdResult<T, E>
    {
        Err(self)
    }
}


