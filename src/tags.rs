use colored_print::color::ConsoleColor;
use colored_print::color::ConsoleColor::*;

pub const TAGS_COLOR: ConsoleColor = LightGreen;
pub const TAGS_ERROR_COLOR: ConsoleColor = Red;
pub const TAGS_INFO_COLOR: ConsoleColor = LightBlue;

pub const COMPILING: &str = "  Compiling";
pub const CREATED: &str = "    Created";
pub const RUNNING: &str = "    Running";
pub const COPYING: &str = "    Copying";
pub const GENERATING: &str = " Generating";
pub const GENERATED: &str = "  Generated";
pub const FINISHED: &str = "   Finished";
pub const FETCHING: &str = "   Fetching";
pub const ERROR: &str = "      Error";
pub const LOGGING_IN: &str = " Logging in";
pub const INFO: &str = "       Info";
pub const DEBUG: &str = "      Debug";
pub const SPACER: &str = "            "; // to indent next line

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

macro_rules! print_copying {
    ($($args:expr),*) => (
        print_with_tag! {
            $crate::tags::TAGS_COLOR, $crate::tags::COPYING, $($args),*
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
    ($enabled:expr, $($args:expr),*) => (
        if $enabled {
            print_with_tag! {
                $crate::tags::TAGS_INFO_COLOR, $crate::tags::INFO, $($args),*
            }
        }
    )
}

macro_rules! print_debug {
    ($enabled:expr, $($args:expr),*) => (
        if cfg!(debug_assertions) && $enabled {
            print_with_tag! {
                $crate::tags::TAGS_INFO_COLOR, $crate::tags::DEBUG, $($args),*
            }
        }
    )
}

macro_rules! print_with_tag {
    ($color:expr, $tag:expr, $fmt:expr $(,$args:expr)*) => (
        colored_eprintln! {
            $crate::imp::common::colorize();
            $color, "{}", $tag;
            $crate::colored_print::color::ConsoleColor::Reset, " ";
            $crate::colored_print::color::ConsoleColor::Reset, $fmt $(,$args)*;
        }
    )
}
