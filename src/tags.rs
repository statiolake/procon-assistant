use colored_print::color::ConsoleColor;
use colored_print::color::ConsoleColor::*;

pub const TAGS_COLOR: ConsoleColor = Green;
pub const TAGS_ERROR_COLOR: ConsoleColor = Red;
pub const TAGS_INFO_COLOR: ConsoleColor = LightBlue;

pub const COMPILING: &str = "  Compiling";
pub const CREATED: &str = "    Created";
pub const RUNNING: &str = "    Running";
pub const GENERATING: &str = " Generating";
pub const GENERATED: &str = "  Generated";
pub const FINISHED: &str = "   Finished";
pub const FETCHING: &str = "   Fetching";
pub const ERROR: &str = "      Error";
pub const LOGGING_IN: &str = " Logging in";
pub const INFO: &str = "       Info";

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

macro_rules! print_compiling {
    ($($args:expr),*) => (
        print_with_tag! {
            $crate::tags::TAGS_COLOR, $crate::tags::COMPILING, $($args),*
        }
    )
}

macro_rules! print_running {
    ($($args:expr),*) => (
        print_with_tag! {
            $crate::tags::TAGS_COLOR, $crate::tags::RUNNING, $($args),*
        }
    )
}

macro_rules! print_generating {
    ($($args:expr),*) => (
        print_with_tag! {
            $crate::tags::TAGS_COLOR, $crate::tags::GENERATING, $($args),*
        }
    )
}

macro_rules! print_generated {
    ($($args:expr),*) => (
        print_with_tag! {
            $crate::tags::TAGS_COLOR, $crate::tags::GENERATED, $($args),*
        }
    )
}

macro_rules! print_fetching {
    ($($args:expr),*) => (
        print_with_tag! {
            $crate::tags::TAGS_COLOR, $crate::tags::FETCHING, $($args),*
        }
    )
}

macro_rules! print_finished {
    ($($args:expr),*) => (
        print_with_tag! {
            $crate::tags::TAGS_COLOR, $crate::tags::FINISHED, $($args),*
        }
    )
}

macro_rules! print_logging_in {
    ($($args:expr),*) => (
        print_with_tag! {
            $crate::tags::TAGS_COLOR, $crate::tags::LOGGING_IN, $($args),*
        }
    )
}

macro_rules! print_info {
    ($($args:expr),*) => (
        print_with_tag! {
            $crate::tags::TAGS_INFO_COLOR, $crate::tags::INFO, $($args),*
        }
    )
}

macro_rules! print_with_tag {
    ($color:expr, $tag:expr, $fmt:expr $(,$args:expr)*) => (
        colored_println! {
            ::common::COLORIZE;
            $color, "{}", $tag;
            ::colored_print::color::ConsoleColor::Reset, " ";
            ::colored_print::color::ConsoleColor::Reset, $fmt $(,$args)*;
        }
    )
}
