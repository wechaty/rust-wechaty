use std::fmt;

// TODO: FileBox Implementation
pub struct FileBox {}

impl fmt::Display for FileBox {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "")
    }
}

impl From<String> for FileBox {
    fn from(_: String) -> Self {
        Self {}
    }
}
