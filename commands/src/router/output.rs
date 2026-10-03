#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    pub text: String,
    pub succeeded: bool,
}

impl Output {
    pub fn success(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            succeeded: true,
        }
    }

    pub fn failure(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            succeeded: false,
        }
    }
}
