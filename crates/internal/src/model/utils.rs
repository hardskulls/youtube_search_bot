use log::Level;
use std::fmt::{Debug, Display};

pub fn log(lvl: Level, op: &str, msg: &str, p: impl Debug) {
    log::log!(lvl, "[LOG]  loc: '{op}'  ( {msg} '{p:?}' )");
}

pub(crate) trait PassWith<T> {
    fn pass_with<R>(self, f: impl FnOnce() -> R) -> T;
}

impl<T> PassWith<T> for T {
    fn pass_with<R>(self, f: impl FnOnce() -> R) -> T {
        f();
        self
    }
}

pub(crate) fn maybe_print<T, P, D>(prefix: P, printable: &Option<T>, default: D) -> String
where
    T: Display + Debug,
    P: Display,
    D: Display,
{
    printable
        .as_ref()
        .map(|p| format!("{prefix}{p:#?}"))
        .unwrap_or_else(|| default.to_string())
}

pub(crate) fn print_if_none<T>(option: Option<T>, text: impl Display) -> String {
    option
        .map(|_| String::new())
        .unwrap_or_else(|| text.to_string())
}

pub(crate) trait HTMLise
where
    Self: Display,
{
    fn to_bold(&self) -> String {
        format!("<b>{self}</b>")
    }

    fn to_link<L: Display>(&self, link_text: L) -> String {
        format!("<a href=\"{self}\">{link_text}</a>")
    }
}

impl<T> HTMLise for T where T: Display {}
