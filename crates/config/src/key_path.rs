pub(crate) const PATH_SEPARATOR: char = '.';

// The nodes a visitor is inside, joined into a key's dotted path.
#[derive(Default)]
pub(crate) struct KeyPath {
    segments: Vec<String>,
}

impl KeyPath {
    pub fn enter(&mut self, name: &str) {
        self.segments.push(name.to_string());
    }

    pub fn leave(&mut self) {
        self.segments.pop();
    }

    pub fn child(&self, name: &str) -> String {
        let mut segments: Vec<&str> = self.segments.iter().map(String::as_str).collect();
        segments.push(name);

        segments.join(&PATH_SEPARATOR.to_string())
    }

    pub fn join(parent: &str, child: &str) -> String {
        match (parent.is_empty(), child.is_empty()) {
            (true, _) => child.to_string(),
            (_, true) => parent.to_string(),
            _ => format!("{parent}{PATH_SEPARATOR}{child}"),
        }
    }

    // The root is the empty path, and holds every key.
    pub fn contains(parent: &str, path: &str) -> bool {
        parent.is_empty()
            || path == parent
            || path
                .strip_prefix(parent)
                .is_some_and(|rest| rest.starts_with(PATH_SEPARATOR))
    }

    pub fn relative<'a>(parent: &str, path: &'a str) -> &'a str {
        if parent.is_empty() {
            return path;
        }

        path.strip_prefix(parent)
            .and_then(|rest| rest.strip_prefix(PATH_SEPARATOR))
            .unwrap_or("")
    }

    pub fn parent(path: &str) -> &str {
        path.rfind(PATH_SEPARATOR).map_or("", |index| &path[..index])
    }
}
