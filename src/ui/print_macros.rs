pub const TAG_WIDTH: usize = 12;

#[doc(hidden)]
#[macro_export]
macro_rules! __eprintln_tagged_impl {
    ($tag:expr, $($args:tt)*) => {
        eprintln!(
            "{:>width$} {}",
            $tag,
            std::format_args!($($args)*),
            width = crate::ui::print_macros::TAG_WIDTH
        );
    };
}

#[macro_export]
macro_rules! eprintln_tagged {
    ($tag:literal: $($args:tt)*) => {
        $crate::__eprintln_tagged_impl!(
            console::style($tag).green().bold(),
            $($args)*
        );
    };
}

#[macro_export]
macro_rules! eprintln_error {
    ($($args:tt)*) => {
        $crate::__eprintln_tagged_impl!(
            console::style("Error").red().bold(),
            $($args)*
        );
    };
}

#[macro_export]
macro_rules! eprintln_warning {
    ($($args:tt)*) => {
        $crate::__eprintln_tagged_impl!(
            console::style("Warning").yellow().bold(),
            $($args)*
        );
    };
}

#[macro_export]
macro_rules! eprintln_info {
    ($($args:tt)*) => {
        $crate::__eprintln_tagged_impl!(
            console::style("Info").cyan().bold(),
            $($args)*
        );
    };
}

#[macro_export]
macro_rules! eprintln_more {
    ($($args:tt)*) => {
        $crate::__eprintln_tagged_impl!(
            ":",
            $($args)*
        );
    };
}
