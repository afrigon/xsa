use kdl::{FormatConfig, KdlDocument, KdlEntry, KdlNode, KdlValue};

const PATH_SEPARATOR: char = '.';

#[derive(Default, Clone)]
pub struct ConfigLayer {
    document: KdlDocument,
}

impl ConfigLayer {
    pub fn new(document: KdlDocument) -> ConfigLayer {
        ConfigLayer { document }
    }

    pub fn document(&self) -> &KdlDocument {
        &self.document
    }

    pub fn get(&self, path: &str) -> Option<&KdlValue> {
        let segments: Vec<&str> = path.split(PATH_SEPARATOR).collect();

        ConfigLayer::get_in(&self.document, &segments)
    }

    // Only the value's own node changes, and only newly created nodes are formatted, so comments and layout
    // elsewhere in the file survive.
    pub fn set(&mut self, path: &str, value: KdlValue) {
        let segments: Vec<&str> = path.split(PATH_SEPARATOR).collect();
        let Some((leaf, parents)) = segments.split_last() else {
            return;
        };
        let mut first_created = None;
        let mut document = &mut self.document;

        for (depth, segment) in parents.iter().enumerate() {
            if document.get(segment).is_none() {
                first_created.get_or_insert(depth);
            }

            document = ConfigLayer::child(document, segment).ensure_children();
        }

        if document.get(leaf).is_none() {
            first_created.get_or_insert(parents.len());
        }

        let node = ConfigLayer::child(document, leaf);

        match node.entries_mut().first_mut() {
            Some(entry) => entry.set_value(value),
            None => node.push(KdlEntry::new(value)),
        }

        if let Some(depth) = first_created {
            self.format_new_node(&segments[..=depth]);
        }
    }

    fn format_new_node(&mut self, segments: &[&str]) {
        let Some((last, parents)) = segments.split_last() else {
            return;
        };
        let mut document = &mut self.document;

        for segment in parents {
            let Some(children) = ConfigLayer::last_mut(document, segment).and_then(|node| node.children_mut().as_mut())
            else {
                return;
            };
            document = children;
        }

        if let Some(node) = ConfigLayer::last_mut(document, last) {
            node.autoformat_config(&FormatConfig::builder().indent_level(parents.len()).build());
        }
    }

    pub fn remove(&mut self, path: &str) {
        let segments: Vec<&str> = path.split(PATH_SEPARATOR).collect();
        ConfigLayer::remove_from(&mut self.document, &segments);
    }

    pub fn leaf_paths(&self) -> Vec<String> {
        let mut paths = Vec::new();
        ConfigLayer::collect_leaves(&self.document, "", &mut paths);

        paths
    }

    // A name can repeat in KDL; like most config formats, the last one wins.
    fn get_in<'a>(document: &'a KdlDocument, segments: &[&str]) -> Option<&'a KdlValue> {
        let (first, rest) = segments.split_first()?;

        document
            .nodes()
            .iter()
            .rev()
            .filter(|node| node.name().value() == *first)
            .find_map(|node| {
                if rest.is_empty() {
                    node.entries().first().map(KdlEntry::value)
                } else {
                    node.children().and_then(|children| ConfigLayer::get_in(children, rest))
                }
            })
    }

    fn child<'a>(document: &'a mut KdlDocument, name: &str) -> &'a mut KdlNode {
        let index = match document.nodes().iter().rposition(|node| node.name().value() == name) {
            Some(index) => index,
            None => {
                document.nodes_mut().push(KdlNode::new(name));
                document.nodes().len() - 1
            }
        };

        &mut document.nodes_mut()[index]
    }

    fn last_mut<'a>(document: &'a mut KdlDocument, name: &str) -> Option<&'a mut KdlNode> {
        document
            .nodes_mut()
            .iter_mut()
            .rev()
            .find(|node| node.name().value() == name)
    }

    // Returns whether the document is empty afterwards, so emptied parent nodes go too.
    fn remove_from(document: &mut KdlDocument, segments: &[&str]) -> bool {
        let Some((first, rest)) = segments.split_first() else {
            return document.nodes().is_empty();
        };

        if rest.is_empty() {
            document.nodes_mut().retain(|node| node.name().value() != *first);
        } else {
            for node in document
                .nodes_mut()
                .iter_mut()
                .filter(|node| node.name().value() == *first)
            {
                if let Some(children) = node.children_mut() {
                    ConfigLayer::remove_from(children, rest);
                }
            }

            document.nodes_mut().retain(|node| {
                node.name().value() != *first || node.children().is_some_and(|children| !children.nodes().is_empty())
            });
        }

        document.nodes().is_empty()
    }

    fn collect_leaves(document: &KdlDocument, prefix: &str, paths: &mut Vec<String>) {
        for node in document.nodes() {
            let path = format!("{prefix}{}", node.name().value());

            match node.children() {
                Some(children) => ConfigLayer::collect_leaves(children, &format!("{path}{PATH_SEPARATOR}"), paths),
                None => paths.push(path),
            }
        }
    }
}
