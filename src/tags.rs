use colored_print::color::ConsoleColor;
use colored_print::color::ConsoleColor::*;

pub const TAGS_COLOR: ConsoleColor = Green;
pub const TAGS_ERROR_COLOR: ConsoleColor = Red;

pub const CREATED: &str = "Created";
pub const   ERROR: &str = "  Error";

macro_rules! print_error {
    ($($args:expr),*) => (
        print_with_tag! {
            $crate::tags::TAGS_ERROR_COLOR, $crate::tags::ERROR, $($args),*
        }
    )
}

macro_rules! print_created {
    ($($args:expr),*) => (
        print_with_tag! {
            $crate::tags::TAGS_COLOR, $crate::tags::CREATED, $($args),*
        }
    )
}

macro_rules! print_with_tag {
    ($color:expr, $tag:expr, $fmt:expr $(,$args:expr)*) => (
        colored_println! {
            true;
            $color, "{}", $tag;
            ::colored_print::color::ConsoleColor::Reset, " ";
            ::colored_print::color::ConsoleColor::Reset, $fmt $(,$args)*;
        }
    )
}
