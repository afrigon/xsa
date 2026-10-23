use std::collections::HashSet;
use std::fmt::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use kdl::KdlValue;

use crate::config_layer::ConfigLayer;
use crate::edit_visitor::{Edit, EditVisitor};
use crate::key_path::{KeyPath, PATH_SEPARATOR};
use crate::key_visitor::KeyVisitor;
use crate::read_visitor::ReadVisitor;
use crate::session_layer::{SessionLayer, SessionValue};
use crate::value_origin::ValueOrigin;
use crate::{ConfigKey, ConfigSection};

const DUMP_INDENT: &str = "    ";

// A value is its session value, else its saved value, else its default. The saved layer is the file in effect.
pub struct ConfigStore<T> {
    path: PathBuf,
    saved: ConfigLayer,
    session: SessionLayer,
    config: T,
    keys: Vec<ConfigKey>,
}

impl<T: ConfigSection + Default + Clone> ConfigStore<T> {
    pub fn load(path: PathBuf) -> anyhow::Result<ConfigStore<T>> {
        let saved = ConfigLayer::read(&path)?;
        tracing::info!("config file in effect: {}", path.display());
        let mut store = ConfigStore {
            path,
            saved,
            session: SessionLayer::default(),
            config: T::default(),
            keys: Vec::new(),
        };
        store.rebuild();
        store.check_unique_keys()?;
        store.warn_unknown_keys(&store.saved);

        Ok(store)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn config(&self) -> &T {
        &self.config
    }

    pub fn keys(&self) -> &[ConfigKey] {
        &self.keys
    }

    pub fn get(&self, path: &str) -> anyhow::Result<String> {
        if let Some(key) = self.key(path) {
            return Ok(key.text.clone());
        }

        self.check_known(path)?;
        bail!("{path} holds several settings: show them with config get --dump {path}")
    }

    // The keys below a path as nested KDL, each value marked with where it comes from.
    pub fn dump(&self, path: &str) -> anyhow::Result<String> {
        let mut text = String::new();
        let mut open: Vec<&str> = Vec::new();

        for key in self.keys_within(path)? {
            let segments: Vec<&str> = key.path.split(PATH_SEPARATOR).collect();
            let (leaf, parents) = segments.split_last().expect("a key path has at least one segment");
            let shared = open
                .iter()
                .zip(parents)
                .take_while(|(open, parent)| open == parent)
                .count();

            while open.len() > shared {
                open.pop();
                let _ = writeln!(text, "{}}}", DUMP_INDENT.repeat(open.len()));
            }

            for parent in &parents[shared..] {
                let _ = writeln!(text, "{}{parent} {{", DUMP_INDENT.repeat(open.len()));
                open.push(parent);
            }

            let origin = self.origin(&key.path);
            let _ = writeln!(
                text,
                "{}{leaf} {} // {origin}",
                DUMP_INDENT.repeat(open.len()),
                key.value
            );
        }

        while !open.is_empty() {
            open.pop();
            let _ = writeln!(text, "{}}}", DUMP_INDENT.repeat(open.len()));
        }

        Ok(text.trim_end().to_string())
    }

    pub fn set(&mut self, path: &str, text: &str, save: bool) -> anyhow::Result<()> {
        let value = self.edit(path, Edit::Parse(text))?;

        self.store(&[(path.to_string(), value)], save)
    }

    pub fn toggle(&mut self, path: &str, save: bool) -> anyhow::Result<()> {
        let value = self.edit(path, Edit::Toggle)?;

        self.store(&[(path.to_string(), value)], save)
    }

    // Every key below the path returns to its default; saved, the keys leave the file.
    pub fn unset(&mut self, path: &str, save: bool) -> anyhow::Result<()> {
        let paths: Vec<String> = self.keys_within(path)?.map(|key| key.path.clone()).collect();

        for path in &paths {
            if save {
                self.saved.remove(path);
            } else {
                self.session.set(path, SessionValue::Unset);
            }
        }

        if save {
            self.session.remove_within(path);
            self.saved.write(&self.path)?;
        }

        self.rebuild();

        Ok(())
    }

    // Writes the saved values below the key into another file, relative to the key; the rest of that file is kept.
    pub fn export(&self, file: &Path, path: &str) -> anyhow::Result<usize> {
        let base = self.base(path);
        let mut target = ConfigLayer::read(file)?;
        let mut count = 0;

        for key in self.keys_within(path)? {
            if let Some(value) = self.saved.get(&key.path) {
                target.set(KeyPath::relative(base, &key.path), value.clone());
                count += 1;
            }
        }

        target.write(file)?;

        Ok(count)
    }

    // Overlays a file written relative to the key onto the session values, or the saved ones.
    pub fn import(&mut self, file: &Path, path: &str, save: bool) -> anyhow::Result<usize> {
        self.check_known(path)?;

        if !file.exists() {
            bail!("{} does not exist", file.display());
        }

        let base = self.base(path).to_string();
        let source = ConfigLayer::read(file)?;
        let mut values = Vec::new();

        for relative in source.leaf_paths() {
            let key_path = KeyPath::join(&base, &relative);

            if !KeyPath::contains(path, &key_path) {
                continue;
            }

            let raw = source.get(&relative).expect("a leaf path holds a value");

            match self.edit(&key_path, Edit::Assign(raw)) {
                Ok(value) => values.push((key_path, value)),
                Err(err) => tracing::warn!("{}: {relative}: {err:#}, ignored", file.display()),
            }
        }

        self.store(&values, save)?;

        Ok(values.len())
    }

    // Reads the file in effect again, below the key.
    pub fn reload(&mut self, path: &str) -> anyhow::Result<()> {
        self.check_known(path)?;
        let file = ConfigLayer::read(&self.path)?;
        self.warn_unknown_keys(&file);

        if path.is_empty() {
            self.saved = file;
        } else {
            self.saved.remove(path);

            for leaf in file.leaf_paths() {
                if KeyPath::contains(path, &leaf) {
                    let value = file.get(&leaf).expect("a leaf path holds a value").clone();
                    self.saved.set(&leaf, value);
                }
            }
        }

        self.rebuild();

        Ok(())
    }

    fn store(&mut self, values: &[(String, KdlValue)], save: bool) -> anyhow::Result<()> {
        for (path, value) in values {
            if save {
                self.saved.set(path, value.clone());
                self.session.remove_within(path);
            } else {
                self.session.set(path, SessionValue::Set(value.clone()));
            }
        }

        if save {
            self.saved.write(&self.path)?;
        }

        self.rebuild();

        Ok(())
    }

    fn edit(&self, path: &str, edit: Edit) -> anyhow::Result<KdlValue> {
        let mut visitor = EditVisitor::new(path, edit);
        self.config.clone().visit(&mut visitor);

        visitor.into_result().with_context(|| format!("config {path}"))
    }

    fn rebuild(&mut self) {
        let mut config = T::default();
        config.visit(&mut ReadVisitor::new(&self.saved, &self.session));
        let mut keys = KeyVisitor::default();
        config.clone().visit(&mut keys);
        self.config = config;
        self.keys = keys.into_keys();
    }

    fn key(&self, path: &str) -> Option<&ConfigKey> {
        self.keys.iter().find(|key| key.path == path)
    }

    fn keys_within(&self, path: &str) -> anyhow::Result<impl Iterator<Item = &ConfigKey>> {
        self.check_known(path)?;

        Ok(self.keys.iter().filter(move |key| KeyPath::contains(path, &key.path)))
    }

    fn check_known(&self, path: &str) -> anyhow::Result<()> {
        if !self.keys.iter().any(|key| KeyPath::contains(path, &key.path)) {
            bail!("unknown config key {path}");
        }

        Ok(())
    }

    // A file for a single key holds it by its own name.
    fn base<'a>(&self, path: &'a str) -> &'a str {
        match self.key(path) {
            Some(_) => KeyPath::parent(path),
            None => path,
        }
    }

    fn origin(&self, path: &str) -> ValueOrigin {
        match self.session.get(path) {
            Some(SessionValue::Set(_)) => ValueOrigin::Session,
            Some(SessionValue::Unset) => ValueOrigin::Default,
            None if self.saved.get(path).is_some() => ValueOrigin::Saved,
            None => ValueOrigin::Default,
        }
    }

    fn check_unique_keys(&self) -> anyhow::Result<()> {
        let mut seen = HashSet::new();

        for key in &self.keys {
            if !seen.insert(&key.path) {
                bail!("config key {} is defined twice", key.path);
            }
        }

        Ok(())
    }

    fn warn_unknown_keys(&self, layer: &ConfigLayer) {
        for path in layer.leaf_paths() {
            if self.key(&path).is_none() {
                tracing::warn!("{}: unknown config key {path}, ignored", self.path.display());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::Duration;

    use super::*;
    use crate::{ConfigChoice, ConfigVisitor, NumberRange};

    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Shade {
        Warm,
        Cold,
    }

    impl ConfigChoice for Shade {
        const ALL: &'static [Shade] = &[Shade::Warm, Shade::Cold];

        fn config_name(self) -> &'static str {
            match self {
                Shade::Warm => "warm",
                Shade::Cold => "cold",
            }
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    struct Glow {
        enabled: bool,
        strength: f32,
    }

    #[derive(Clone, Debug, PartialEq)]
    struct Settings {
        shade: Option<Shade>,
        stars: bool,
        glow: Glow,
        delay: Option<Duration>,
    }

    impl Default for Settings {
        fn default() -> Settings {
            Settings {
                shade: Some(Shade::Warm),
                stars: true,
                glow: Glow {
                    enabled: true,
                    strength: 0.02,
                },
                delay: None,
            }
        }
    }

    impl ConfigSection for Glow {
        fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
            visitor.value("enabled", &mut self.enabled);
            visitor.number("strength", &mut self.strength, NumberRange::at_least(0.0));
        }
    }

    impl ConfigSection for Settings {
        fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
            visitor.group("video", |visitor| {
                visitor.value("shade", &mut self.shade);
                visitor.value("stars", &mut self.stars);
                visitor.section("glow", &mut self.glow);
            });
            visitor.value("delay", &mut self.delay);
        }
    }

    struct TestDirectory {
        path: PathBuf,
    }

    impl TestDirectory {
        fn new(name: &str) -> TestDirectory {
            let path = std::env::temp_dir().join(format!("xsa-config-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).unwrap();

            TestDirectory { path }
        }

        fn file(&self, name: &str, contents: Option<&str>) -> PathBuf {
            let path = self.path.join(name);

            if let Some(contents) = contents {
                fs::write(&path, contents).unwrap();
            }

            path
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn store(directory: &TestDirectory, contents: Option<&str>) -> ConfigStore<Settings> {
        ConfigStore::load(directory.file("config.kdl", contents)).unwrap()
    }

    #[test]
    fn a_missing_file_gives_the_defaults() {
        let directory = TestDirectory::new("missing");
        assert_eq!(store(&directory, None).config(), &Settings::default());
    }

    #[test]
    fn file_values_override_the_defaults_and_a_repeated_block_is_read_last_wins() {
        let directory = TestDirectory::new("file");
        let store = store(
            &directory,
            Some("video { shade \"cold\"; stars #false }\nvideo { shade \"none\" }\n"),
        );
        assert_eq!(store.config().shade, None);
        assert!(!store.config().stars);
        assert_eq!(store.config().glow, Settings::default().glow);
    }

    #[test]
    fn invalid_values_fall_back_to_the_default() {
        let directory = TestDirectory::new("invalid");
        let store = store(&directory, Some("video { shade \"sepia\"; glow { strength -1 } }\n"));
        assert_eq!(store.config(), &Settings::default());
    }

    #[test]
    fn a_session_value_applies_but_is_never_written() {
        let directory = TestDirectory::new("session");
        let mut store = store(&directory, None);
        store.set("video.stars", "off", false).unwrap();
        assert!(!store.config().stars);
        assert!(!store.path().exists());
    }

    #[test]
    fn saving_keeps_comments_and_writes_only_what_was_set_even_a_default() {
        let directory = TestDirectory::new("save");
        let mut store = store(
            &directory,
            Some("// my settings\nvideo {\n    stars #false // dark sky\n}\n"),
        );
        store.set("video.glow.enabled", "false", true).unwrap();
        store.set("video.shade", "warm", true).unwrap();
        let contents = fs::read_to_string(store.path()).unwrap();
        assert!(
            contents.contains("// my settings") && contents.contains("// dark sky"),
            "{contents}"
        );
        assert!(contents.contains("enabled #false"), "{contents}");
        assert!(
            contents.contains("shade warm") || contents.contains("shade \"warm\""),
            "{contents}"
        );
        assert!(!contents.contains("delay"), "{contents}");
    }

    #[test]
    fn a_saved_value_replaces_the_session_one() {
        let directory = TestDirectory::new("replace");
        let mut store = store(&directory, None);
        store.set("video.stars", "false", false).unwrap();
        store.set("video.stars", "true", true).unwrap();
        assert!(store.config().stars);
        assert_eq!(
            store.dump("video.stars").unwrap(),
            "video {\n    stars #true // saved\n}"
        );
    }

    #[test]
    fn unsetting_returns_to_the_default_and_saved_removes_the_key() {
        let directory = TestDirectory::new("unset");
        let mut store = store(&directory, Some("video { stars #false }\n"));
        store.unset("video.stars", false).unwrap();
        assert!(store.config().stars);
        assert!(fs::read_to_string(store.path()).unwrap().contains("stars"));

        store.unset("video", true).unwrap();
        assert!(store.config().stars);
        assert!(!fs::read_to_string(store.path()).unwrap().contains("stars"));
    }

    #[test]
    fn toggling_cycles_choices_and_flips_switches() {
        let directory = TestDirectory::new("toggle");
        let mut store = store(&directory, None);
        store.toggle("video.shade", false).unwrap();
        store.toggle("video.stars", false).unwrap();
        assert_eq!(store.config().shade, Some(Shade::Cold));
        assert!(!store.config().stars);
        store.toggle("video.shade", false).unwrap();
        assert_eq!(store.config().shade, None);
        assert!(store.toggle("video.glow.strength", false).is_err());
    }

    #[test]
    fn values_outside_their_range_or_type_are_rejected() {
        let directory = TestDirectory::new("range");
        let mut store = store(&directory, None);
        assert!(store.set("video.glow.strength", "-1", false).is_err());
        assert!(store.set("video.stars", "maybe", false).is_err());
        assert!(store.set("video.nothing", "1", false).is_err());
        store.set("delay", "50ms", false).unwrap();
        assert_eq!(store.config().delay, Some(Duration::from_millis(50)));
    }

    #[test]
    fn get_shows_one_value_and_dump_a_subtree_with_origins() {
        let directory = TestDirectory::new("dump");
        let mut store = store(&directory, Some("video { stars #false }\n"));
        store.set("video.glow.strength", "0.5", false).unwrap();
        assert_eq!(store.get("video.shade").unwrap(), "warm");
        assert!(store.get("video").is_err());
        assert_eq!(
            store.dump("video").unwrap(),
            "video {\n    shade warm // default\n    stars #false // saved\n    glow {\n        enabled #true // default\n        strength 0.5 // session\n    }\n}"
        );
    }

    #[test]
    fn export_writes_saved_values_relative_to_the_key_and_import_reads_them_back() {
        let directory = TestDirectory::new("export");
        let mut store = store(&directory, Some("video { stars #false; glow { strength 0.5 } }\n"));
        store.set("video.shade", "cold", false).unwrap();
        let backup = directory.file("backup.kdl", None);
        assert_eq!(store.export(&backup, "video.glow").unwrap(), 1);
        assert_eq!(fs::read_to_string(&backup).unwrap().trim(), "strength 0.5");
        assert_eq!(store.export(&backup, "video").unwrap(), 2);
        assert!(!fs::read_to_string(&backup).unwrap().contains("shade"));

        let mut other = ConfigStore::<Settings>::load(directory.file("other.kdl", None)).unwrap();
        assert_eq!(other.import(&backup, "video.glow", false).unwrap(), 1);
        assert_eq!(other.config().glow.strength, 0.5);
        assert!(other.config().stars);
        assert_eq!(other.import(&backup, "video", false).unwrap(), 2);
        assert_eq!(other.config().glow.strength, 0.5);
        assert!(!other.path().exists());
        other.import(&backup, "video", true).unwrap();
        assert!(fs::read_to_string(other.path()).unwrap().contains("strength 0.5"));
    }

    #[test]
    fn reloading_reads_the_file_again_below_the_key() {
        let directory = TestDirectory::new("reload");
        let mut store = store(&directory, Some("video { stars #false }\ndelay \"1s\"\n"));
        fs::write(store.path(), "video { stars #true }\n").unwrap();
        store.reload("video").unwrap();
        assert!(store.config().stars);
        assert_eq!(store.config().delay, Some(Duration::from_secs(1)));
        store.reload("").unwrap();
        assert_eq!(store.config().delay, None);
    }
}
