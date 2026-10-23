use kdl::KdlValue;

#[derive(Clone, Debug)]
pub struct ConfigKey {
    pub path: String,
    pub value: KdlValue,
    pub text: String,
    pub completions: Vec<String>,
}
