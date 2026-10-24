use kdl::KdlValue;

#[derive(Clone, Debug)]
pub struct ConfigKey {
    pub path: String,
    pub value: KdlValue,
    pub text: String,
    pub completions: Vec<String>,
    pub type_name: String,
    // What the schema notes beside the type: `optional`, `inheritable`, a range.
    pub qualifiers: Vec<String>,
}
