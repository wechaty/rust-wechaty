use std::fmt;

// TODO: FileBox Implementation
pub struct FileBox {}

impl FileBox {
    pub fn to_string(&self) -> String {
        String::new()
    }
}

impl fmt::Display for FileBox {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", self.to_string())
    }
}

impl From<String> for FileBox {
    fn from(_: String) -> Self {
        Self {}
    }
}
