use std::fmt::{Debug, Display};

pub(crate) fn maybe_print<T: Display + Debug, P: Display, D: Display>(prefix: P, printable: &Option<T>, default: D) -> String
{
    if let Some(p) = printable
    { format!("{prefix}{p:#?}") }
    else
    { default.to_string() }
}

pub(crate) fn print_if_none<T>(option: Option<T>, text: impl Display) -> String
{
    if option.is_none()
    { text.to_string() }
    else
    { "".to_string() }
}

pub(crate) trait HTMLise
    where
        Self: Display
{
    fn to_bold(&self) -> String
    {
        format!("<b>{self}</b>")
    }
    
    fn to_link<L: Display>(&self, link_text: L) -> String
    {
        format!("<a href=\"{self}\">{link_text}</a>")
    }
}

impl<T> HTMLise for T
    where T: Display
{}


