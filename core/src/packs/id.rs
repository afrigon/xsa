use std::fmt;

use anyhow::ensure;

pub const NAMESPACE_SEPARATOR: char = ':';

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Id {
    pub namespace: String,
    pub path: String,
}

impl Id {
    pub fn parse(text: &str, default_namespace: &str) -> anyhow::Result<Id> {
        let id = match text.split_once(NAMESPACE_SEPARATOR) {
            Some((namespace, path)) => Id {
                namespace: namespace.to_string(),
                path: path.to_string(),
            },
            None => Id {
                namespace: default_namespace.to_string(),
                path: text.to_string(),
            },
        };
        ensure!(
            !id.namespace.is_empty() && !id.path.is_empty() && !id.path.contains(NAMESPACE_SEPARATOR),
            "{text:?} is not an identifier like namespace:path"
        );
        Ok(id)
    }
}

impl fmt::Display for Id {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}{NAMESPACE_SEPARATOR}{}", self.namespace, self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_namespace_means_the_default() {
        let id = Id::parse("earth", "system-solar").unwrap();
        assert_eq!(id.to_string(), "system-solar:earth");
    }

    #[test]
    fn paths_may_contain_folders() {
        let id = Id::parse("system-solar:skyboxes/deep-star-maps", "base").unwrap();
        assert_eq!(id.namespace, "system-solar");
        assert_eq!(id.path, "skyboxes/deep-star-maps");
    }

    #[test]
    fn empty_parts_are_rejected() {
        assert!(Id::parse(":earth", "base").is_err());
        assert!(Id::parse("base:", "base").is_err());
        assert!(Id::parse("a:b:c", "base").is_err());
    }
}
