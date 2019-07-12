use std::error;

macro_rules! define_error {
    () => {
        pub type Result<T> = std::result::Result<T, Error>;

        #[derive(Debug)]
        pub struct Error {
            pub kind: ErrorKind,
            pub cause: Option<Box<std::error::Error + Send>>,
        }

        pub trait ChainableToError<T> {
            fn chain(self, kind: ErrorKind) -> Result<T>;
        }

        impl Error {
            #[allow(dead_code)]
            pub fn new(kind: ErrorKind) -> Error {
                Error { kind, cause: None }
            }

            #[allow(dead_code)]
            pub fn with_cause(kind: ErrorKind, cause: Box<std::error::Error + Send>) -> Error {
                Error {
                    kind,
                    cause: Some(cause),
                }
            }
        }

        impl std::error::Error for Error {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                self.cause.as_ref().map(|e| &**e as &std::error::Error)
            }
        }

        impl crate::ErrorWithSilent for Error {
            fn is_silent(&self) -> bool {
                match self.kind {
                    ErrorKind::SilentError => true,
                    _ => false,
                }
            }

            fn upcast(&self) -> &(dyn std::error::Error + Send) {
                self
            }
        }

        impl<T, E: 'static + std::error::Error + Send> ChainableToError<T>
            for std::result::Result<T, E>
        {
            fn chain(self, kind: ErrorKind) -> Result<T> {
                self.map_err(|e| Error::with_cause(kind, Box::new(e)))
            }
        }
    };
}

macro_rules! define_error_kind {
    () => {
        #[derive(Debug)]
        pub enum ErrorKind {
            #[allow(dead_code)] SilentError,
        }
        impl std::fmt::Display for ErrorKind {
            fn fmt(&self, _: &mut std::fmt::Formatter) -> std::fmt::Result {
                unreachable!("SilentError must not be formatted.");
            }
        }
    };
    ($([$id:ident; ($($cap:ident : $ty:ty),*); $ex:expr];)*) => {
        #[derive(Debug)]
        pub enum ErrorKind {
            /// quietly stops the application with error state. this is used in
            /// `run` command, when some test case fails. no other error
            /// messages are needed, but run command should return error value
            /// when some test fails, so that other utilities can get the test
            /// result.
            #[allow(dead_code)] SilentError,
            $($id($($ty),*)),*
        }

        impl std::fmt::Display for Error {
            fn fmt(&self, b: &mut std::fmt::Formatter) -> std::fmt::Result {
                let message = match self.kind {
                    // do nothing when SilentError
                    ErrorKind::SilentError => unreachable!("SilentError must not be formatted."),
                    $(ErrorKind::$id($(ref $cap),*) => $ex),*
                };
                write!(b, "{}", message)
            }
        }
    };
}

trait ErrorWithSilent: error::Error + Send {
    fn is_silent(&self) -> bool;
    fn upcast(&self) -> &(dyn error::Error + Send);
}

pub mod imp;
pub mod ui;

fn main() {
    ui::main()
}
