use colored_print::color::{ConsoleColor, ConsoleColor::*};

pub const TAGS_COLOR: ConsoleColor = LightGreen;
pub const TAGS_ERROR_COLOR: ConsoleColor = Red;
pub const TAGS_WARNING_COLOR: ConsoleColor = Yellow;
pub const TAGS_INFO_COLOR: ConsoleColor = Cyan;

pub const COMPILING: &str = "  Compiling";
pub const CREATED: &str = "    Created";
pub const RUNNING: &str = "    Running";
pub const COPYING: &str = "    Copying";
pub const GENERATING: &str = " Generating";
pub const GENERATED: &str = "  Generated";
pub const FINISHED: &str = "   Finished";
pub const FETCHING: &str = "   Fetching";
pub const ERROR: &str = "      Error";
pub const WARNING: &str = "    Warning";
pub const LOGGING_IN: &str = " Logging in";
pub const LINT: &str = "       Lint";
pub const INFO: &str = "       Info";
pub const DEBUG: &str = "      Debug";

#[macro_export]
macro_rules! print_error {
    ($($args:expr),*) => (
        $crate::print_with_tag! {
            $crate::ui::tags::TAGS_ERROR_COLOR, $crate::ui::tags::ERROR, $($args),*
        }
    )
}

#[macro_export]
macro_rules! print_warning {
    ($($args:expr),*) => (
        $crate::print_with_tag! {
            $crate::ui::tags::TAGS_WARNING_COLOR, $crate::ui::tags::WARNING, $($args),*
        }
    )
}

#[macro_export]
macro_rules! print_created {
    ($($args:expr),*) => (
        $crate::print_with_tag! {
            $crate::ui::tags::TAGS_COLOR, $crate::ui::tags::CREATED, $($args),*
        }
    )
}

#[macro_export]
macro_rules! print_compiling {
    ($($args:expr),*) => (
        $crate::print_with_tag! {
            $crate::ui::tags::TAGS_COLOR, $crate::ui::tags::COMPILING, $($args),*
        }
    )
}

#[macro_export]
macro_rules! print_running {
    ($($args:expr),*) => (
        $crate::print_with_tag! {
            $crate::ui::tags::TAGS_COLOR, $crate::ui::tags::RUNNING, $($args),*
        }
    )
}

#[macro_export]
macro_rules! print_copying {
    ($($args:expr),*) => (
        $crate::print_with_tag! {
            $crate::ui::tags::TAGS_COLOR, $crate::ui::tags::COPYING, $($args),*
        }
    )
}

#[macro_export]
macro_rules! print_generating {
    ($($args:expr),*) => (
        $crate::print_with_tag! {
            $crate::ui::tags::TAGS_COLOR, $crate::ui::tags::GENERATING, $($args),*
        }
    )
}

#[macro_export]
macro_rules! print_generated {
    ($($args:expr),*) => (
        $crate::print_with_tag! {
            $crate::ui::tags::TAGS_COLOR, $crate::ui::tags::GENERATED, $($args),*
        }
    )
}

#[macro_export]
macro_rules! print_fetching {
    ($($args:expr),*) => (
        $crate::print_with_tag! {
            $crate::ui::tags::TAGS_COLOR, $crate::ui::tags::FETCHING, $($args),*
        }
    )
}

#[macro_export]
macro_rules! print_finished {
    ($($args:expr),*) => (
        $crate::print_with_tag! {
            $crate::ui::tags::TAGS_COLOR, $crate::ui::tags::FINISHED, $($args),*
        }
    )
}

#[macro_export]
macro_rules! print_logging_in {
    ($($args:expr),*) => (
        $crate::print_with_tag! {
            $crate::ui::tags::TAGS_COLOR, $crate::ui::tags::LOGGING_IN, $($args),*
        }
    )
}

#[macro_export]
macro_rules! print_lint {
    ($($args:expr),*) => (
        $crate::print_with_tag! {
            $crate::ui::tags::TAGS_INFO_COLOR, $crate::ui::tags::LINT, $($args),*
        }
    )
}

#[macro_export]
macro_rules! print_info {
    ($enabled:expr, $($args:expr),*) => (
        if $enabled {
            $crate::print_with_tag! {
                $crate::ui::tags::TAGS_INFO_COLOR, $crate::ui::tags::INFO, $($args),*
            }
        }
    )
}

#[macro_export]
macro_rules! print_debug {
    ($enabled:expr, $($args:expr),*) => (
        if cfg!(debug_assertions) && $enabled {
            $crate::print_with_tag! {
                $crate::ui::tags::TAGS_INFO_COLOR, $crate::ui::tags::DEBUG, $($args),*
            }
        }
    )
}

#[macro_export]
macro_rules! print_with_tag {
    ($color:expr, $tag:expr, $fmt:expr $(,$args:expr)*) => (
        colored_print::colored_eprintln! {
            $crate::imp::common::colorize();
            $color, "{}", $tag;
            colored_print::color::ConsoleColor::Reset, " ";
            colored_print::color::ConsoleColor::Reset, $fmt $(,$args)*;
        }
    )
}
