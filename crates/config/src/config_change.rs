// A key and the text of its new value, one of several set together.
#[derive(Clone, Copy, Debug)]
pub struct ConfigChange<'a> {
    pub path: &'a str,
    pub text: &'a str,
}
