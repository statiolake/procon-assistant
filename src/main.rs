macro_rules! delegate_impl_error_error_kind {
    (#[error($tag:literal)] pub struct $error:ident($errorkind:ty);) => {
        #[derive(Debug)]
        pub struct $error($errorkind);

        impl std::error::Error for $error {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                self.0.source()
            }
        }

        impl std::fmt::Display for $error {
            fn fmt(&self, b: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(b, concat!($tag, ": {}"), self.0)
            }
        }
    };
}

pub mod imp;
pub mod ui;

fn main() {
    ui::main()
}
