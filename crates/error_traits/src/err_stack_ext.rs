use std::fmt::{Debug, Display};
use error_stack::{Context, IntoReportCompat, ResultExt};

/// Conversion and adjustment of error reports (after turning usual result to
/// report by calling `.into_report()`).
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

impl<T, C> ConvReport for error_stack::Result<T, C> {}


/// Create report from usual result ( required for `Box(dyn Error)` ).
pub trait IntoReportDyn
{
    type Ok;
    type Err;
    
    fn into_report_dyn(self) -> error_stack::Result<Self::Ok, Self::Err>;
}

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


