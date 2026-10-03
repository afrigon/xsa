pub fn split(line: &str) -> Result<Vec<String>, String> {
    shlex::split(line).ok_or_else(|| "unclosed quote or trailing backslash".to_string())
}
