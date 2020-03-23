use console::Style;

pub fn tags_color() -> Style {
    Style::new().green().bold()
}

pub fn tags_error_color() -> Style {
    Style::new().red()
}

pub fn tags_warning_color() -> Style {
    Style::new().yellow()
}

pub fn tags_info_color() -> Style {
    Style::new().cyan()
}

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
            $crate::ui::tags::tags_error_color, $crate::ui::tags::ERROR, $($args),*
        }
    )
}

#[macro_export]
macro_rules! print_warning {
    ($($args:expr),*) => (
        $crate::print_with_tag! {
            $crate::ui::tags::tags_warning_color, $crate::ui::tags::WARNING, $($args),*
        }
    )
}

#[macro_export]
macro_rules! print_created {
    ($($args:expr),*) => (
        $crate::print_with_tag! {
            $crate::ui::tags::tags_color, $crate::ui::tags::CREATED, $($args),*
        }
    )
}

#[macro_export]
macro_rules! print_compiling {
    ($($args:expr),*) => (
        $crate::print_with_tag! {
            $crate::ui::tags::tags_color, $crate::ui::tags::COMPILING, $($args),*
        }
    )
}

#[macro_export]
macro_rules! print_running {
    ($($args:expr),*) => (
        $crate::print_with_tag! {
            $crate::ui::tags::tags_color, $crate::ui::tags::RUNNING, $($args),*
        }
    )
}

#[macro_export]
macro_rules! print_copying {
    ($($args:expr),*) => (
        $crate::print_with_tag! {
            $crate::ui::tags::tags_color, $crate::ui::tags::COPYING, $($args),*
        }
    )
}

#[macro_export]
macro_rules! print_generating {
    ($($args:expr),*) => (
        $crate::print_with_tag! {
            $crate::ui::tags::tags_color, $crate::ui::tags::GENERATING, $($args),*
        }
    )
}

#[macro_export]
macro_rules! print_generated {
    ($($args:expr),*) => (
        $crate::print_with_tag! {
            $crate::ui::tags::tags_color, $crate::ui::tags::GENERATED, $($args),*
        }
    )
}

#[macro_export]
macro_rules! print_fetching {
    ($($args:expr),*) => (
        $crate::print_with_tag! {
            $crate::ui::tags::tags_color, $crate::ui::tags::FETCHING, $($args),*
        }
    )
}

#[macro_export]
macro_rules! print_finished {
    ($($args:expr),*) => (
        $crate::print_with_tag! {
            $crate::ui::tags::tags_color, $crate::ui::tags::FINISHED, $($args),*
        }
    )
}

#[macro_export]
macro_rules! print_logging_in {
    ($($args:expr),*) => (
        $crate::print_with_tag! {
            $crate::ui::tags::tags_color, $crate::ui::tags::LOGGING_IN, $($args),*
        }
    )
}

#[macro_export]
macro_rules! print_lint {
    ($($args:expr),*) => (
        $crate::print_with_tag! {
            $crate::ui::tags::tags_info_color, $crate::ui::tags::LINT, $($args),*
        }
    )
}

#[macro_export]
macro_rules! print_info {
    ($enabled:expr, $($args:expr),*) => (
        if $enabled {
            $crate::print_with_tag! {
                $crate::ui::tags::tags_info_color, $crate::ui::tags::INFO, $($args),*
            }
        }
    )
}

#[macro_export]
macro_rules! print_debug {
    ($enabled:expr, $($args:expr),*) => (
        if cfg!(debug_assertions) && $enabled {
            $crate::print_with_tag! {
                $crate::ui::tags::tags_info_color, $crate::ui::tags::DEBUG, $($args),*
            }
        }
    )
}

#[macro_export]
macro_rules! print_with_tag {
    ($color:expr, $tag:expr, $fmt:expr $(,$args:expr)*) => (
        eprintln!("{} {}", $color().apply_to($tag), format_args!($fmt $(,$args)*));
    )
}
