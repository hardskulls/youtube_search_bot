use std::fmt::Display;

pub fn print_if_none<T>(option: Option<T>, text: impl Display) -> String
{
    if option.is_none()
    { text.to_string() }
    else
    { "".to_string() }
}

pub trait HTMLise
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


