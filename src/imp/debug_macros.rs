#[macro_export]
macro_rules! eprintln_debug {
    ($($args:tt)*) => {
        if cfg!(debug_assertions) {
            $crate::__eprintln_tagged_impl!(console::style("Debug").magenta(), $($args)*);
        }
    };
}

#[macro_export]
macro_rules! eprintln_debug_more {
    ($($args:tt)*) => {
        if cfg!(debug_assertions) {
            $crate::eprintln_more!($($args)*);
        }
    };
}
