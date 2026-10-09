mod adaptation_document;
mod animation_document;
mod antialiasing_document;
mod antialiasing_kind_choice;
mod bind_document;
mod bloom_document;
mod config_choice;
mod config_key;
mod config_layer;
mod config_value_kind;
mod config_values;
mod exposure_document;
mod exposure_mode_choice;
mod key_chord_name;
mod number_range;
mod render_document;
mod shader_choice;
mod shading_model_choice;
mod tonemapper_choice;

pub use antialiasing_document::AntialiasingDocument;
pub use bloom_document::BloomDocument;
pub use config_choice::ConfigChoice;
pub use config_key::ConfigKey;
pub use config_value_kind::ConfigValueKind;
pub use exposure_document::ExposureDocument;
pub use number_range::NumberRange;
pub use render_document::RenderDocument;

use adaptation_document::AdaptationDocument;
use animation_document::AnimationDocument;
use bind_document::BindDocument;
use config_layer::ConfigLayer;
use config_values::ConfigValues;

use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use kdl::{KdlDocument, KdlValue};

use crate::config::Config;

const DIRECTORY_NAME: &str = "xsa";
const FILE_NAME: &str = "config.kdl";

// Three layers: values of keys that are never saved, the file, and the defaults beneath both.
pub struct ConfigDocument {
    path: PathBuf,
    session: ConfigLayer,
    file: ConfigLayer,
    defaults: ConfigLayer,
}

impl ConfigDocument {
    pub fn default_path() -> anyhow::Result<PathBuf> {
        let directory = match env::var_os("XDG_CONFIG_HOME").filter(|value| !value.is_empty()) {
            Some(directory) => PathBuf::from(directory),
            None if cfg!(windows) => PathBuf::from(env::var_os("APPDATA").context("APPDATA is not set")?),
            None => PathBuf::from(env::var_os("HOME").context("HOME is not set")?).join(".config"),
        };

        Ok(directory.join(DIRECTORY_NAME).join(FILE_NAME))
    }

    pub fn load(path: PathBuf) -> anyhow::Result<ConfigDocument> {
        let mut defaults = ConfigLayer::default();
        let config = Config::default();
        RenderDocument::write(&config.render, &mut defaults);
        BindDocument::write(&config.bind, &mut defaults);
        AnimationDocument::write(&config.animations, &mut defaults);
        let mut document = ConfigDocument {
            path,
            session: ConfigLayer::default(),
            file: ConfigLayer::default(),
            defaults,
        };
        document.reload()?;

        Ok(document)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn reload(&mut self) -> anyhow::Result<()> {
        let text = match fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(err) if err.kind() == ErrorKind::NotFound => String::new(),
            Err(err) => return Err(err).with_context(|| format!("reading {}", self.path.display())),
        };
        let document = KdlDocument::parse(&text).with_context(|| format!("parsing {}", self.path.display()))?;
        self.file = ConfigLayer::new(document);

        for path in self.file.leaf_paths() {
            match ConfigKey::find(&path) {
                None => tracing::warn!("{}: unknown config key {path}, ignored", self.path.display()),
                Some(key) if !key.persisted => {
                    tracing::warn!(
                        "{}: {path} cannot be set in the config file, ignored",
                        self.path.display()
                    );
                }
                Some(_) => {}
            }
        }

        Ok(())
    }

    pub fn config(&self) -> Config {
        let values = ConfigValues {
            session: &self.session,
            file: &self.file,
        };
        let defaults = Config::default();

        Config {
            render: RenderDocument::read(&values).into_config(&defaults.render),
            bind: BindDocument::read(&values).into_config(&defaults.bind),
            animations: AnimationDocument::read(&values).into_config(&defaults.animations),
        }
    }

    // A key prints its value; a prefix such as `render.exposure` prints every key below it.
    pub fn get(&self, path: &str) -> anyhow::Result<String> {
        if let Some(key) = ConfigKey::find(path) {
            return Ok(ConfigValueKind::text(self.value(key)));
        }

        let prefix = format!("{path}.");
        let lines: Vec<String> = ConfigKey::all()
            .into_iter()
            .filter(|key| path.is_empty() || key.path.starts_with(&prefix))
            .map(|key| format!("{} {}", key.path, ConfigValueKind::text(self.value(key))))
            .collect();

        if lines.is_empty() {
            bail!("unknown config key {path}");
        }

        Ok(lines.join("\n"))
    }

    pub fn set(&mut self, path: &str, text: &str) -> anyhow::Result<()> {
        let key = ConfigDocument::key(path)?;
        let value = key.kind.parse(text)?;
        self.store(key, value);

        Ok(())
    }

    pub fn toggle(&mut self, path: &str) -> anyhow::Result<()> {
        let key = ConfigDocument::key(path)?;
        let value = key.kind.toggled(self.value(key))?;
        self.store(key, value);

        Ok(())
    }

    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(directory) = self.path.parent() {
            fs::create_dir_all(directory).with_context(|| format!("creating {}", directory.display()))?;
        }

        fs::write(&self.path, self.file.document().to_string())
            .with_context(|| format!("writing {}", self.path.display()))
    }

    fn key(path: &str) -> anyhow::Result<ConfigKey> {
        ConfigKey::find(path).with_context(|| format!("unknown config key {path}"))
    }

    fn value(&self, key: ConfigKey) -> &KdlValue {
        let file = key.persisted.then_some(&self.file);

        [Some(&self.session), file, Some(&self.defaults)]
            .into_iter()
            .flatten()
            .find_map(|layer| layer.get(key.path))
            .expect("every key has a default")
    }

    // A default is removed rather than written, so the file holds only what was changed.
    fn store(&mut self, key: ConfigKey, value: KdlValue) {
        let layer = if key.persisted {
            &mut self.file
        } else {
            &mut self.session
        };
        let is_default = self
            .defaults
            .get(key.path)
            .is_some_and(|default| key.kind.same(default, &value));

        if is_default {
            layer.remove(key.path);
        } else {
            layer.set(key.path, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::Tonemapper;

    struct TestFile {
        directory: PathBuf,
    }

    impl TestFile {
        fn new(name: &str, contents: Option<&str>) -> TestFile {
            let directory = env::temp_dir().join(format!("xsa-config-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&directory);
            fs::create_dir_all(&directory).unwrap();
            let file = TestFile { directory };

            if let Some(contents) = contents {
                fs::write(file.path(), contents).unwrap();
            }

            file
        }

        fn path(&self) -> PathBuf {
            self.directory.join(FILE_NAME)
        }

        fn contents(&self) -> String {
            fs::read_to_string(self.path()).unwrap()
        }
    }

    impl Drop for TestFile {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.directory);
        }
    }

    #[test]
    fn a_missing_file_gives_the_defaults() {
        let file = TestFile::new("missing", None);
        let document = ConfigDocument::load(file.path()).unwrap();
        assert_eq!(document.config(), Config::default());
    }

    #[test]
    fn file_values_override_the_defaults() {
        let file = TestFile::new(
            "override",
            Some("render {\n    tonemapper \"off\"\n    stars #false\n}\n"),
        );
        let config = ConfigDocument::load(file.path()).unwrap().config();
        assert_eq!(config.render.tonemapper, Tonemapper::Off);
        assert!(!config.render.stars);
        assert_eq!(config.render.bloom, Config::default().render.bloom);
    }

    #[test]
    fn saving_keeps_comments_and_writes_only_changes() {
        let file = TestFile::new(
            "comments",
            Some("// my settings\nrender {\n    stars #false // dark sky\n}\n"),
        );
        let mut document = ConfigDocument::load(file.path()).unwrap();
        document.set("render.bloom.enabled", "false").unwrap();
        document.save().unwrap();
        let contents = file.contents();
        assert!(
            contents.contains("// my settings") && contents.contains("// dark sky"),
            "{contents}"
        );
        assert!(contents.contains("enabled #false"), "{contents}");
        assert!(!contents.contains("tonemapper"), "{contents}");
    }

    #[test]
    fn setting_a_default_removes_it_from_the_file() {
        let file = TestFile::new(
            "default",
            Some("render {\n    bloom {\n        enabled #false\n    }\n}\n"),
        );
        let mut document = ConfigDocument::load(file.path()).unwrap();
        document.set("render.bloom.enabled", "true").unwrap();
        document.save().unwrap();
        assert!(!file.contents().contains("bloom"), "{}", file.contents());
    }

    #[test]
    fn a_repeated_block_is_read_last_wins() {
        let file = TestFile::new(
            "repeated",
            Some("render { stars #false; tonemapper \"agx-punchy\" }\nrender { tonemapper \"off\" }\n"),
        );
        let mut document = ConfigDocument::load(file.path()).unwrap();
        let config = document.config();
        assert!(!config.render.stars);
        assert_eq!(config.render.tonemapper, Tonemapper::Off);

        document.set("render.tonemapper", "agx").unwrap();
        assert_eq!(document.config().render.tonemapper, Tonemapper::Agx);
    }

    #[test]
    fn binds_with_symbol_keys_survive_a_save() {
        let file = TestFile::new("binds", None);
        let mut document = ConfigDocument::load(file.path()).unwrap();
        let binds = [
            "bind.camera.target-next",
            "bind.camera.target-previous",
            "bind.interface.pause-menu",
            "bind.interface.debug-overlay",
        ];
        let keys = ["\\", "=", "[", "ctrl+'"];

        for (bind, key) in binds.iter().zip(keys) {
            document.set(bind, key).unwrap();
        }

        document.save().unwrap();
        let reloaded = ConfigDocument::load(file.path()).unwrap();

        for (bind, key) in binds.iter().zip(keys) {
            assert_eq!(reloaded.get(bind).unwrap(), key, "{}", file.contents());
        }

        assert_eq!(reloaded.config().bind, document.config().bind);
    }

    #[test]
    fn invalid_values_fall_back_to_the_default() {
        let file = TestFile::new("invalid", Some("render { tonemapper \"sepia\"; stars 3 }\n"));
        let config = ConfigDocument::load(file.path()).unwrap().config();
        assert_eq!(config.render, Config::default().render);
    }

    #[test]
    fn the_file_cannot_set_session_settings() {
        let file = TestFile::new("session", Some("render { wireframe #true }\n"));
        let document = ConfigDocument::load(file.path()).unwrap();
        assert!(!document.config().render.wireframe);
        assert_eq!(document.get("render.wireframe").unwrap(), "false");
    }

    #[test]
    fn session_settings_apply_but_are_never_saved() {
        let file = TestFile::new("session-save", None);
        let mut document = ConfigDocument::load(file.path()).unwrap();
        document.set("render.wireframe", "true").unwrap();
        assert!(document.config().render.wireframe);
        document.save().unwrap();
        assert!(!file.contents().contains("wireframe"), "{}", file.contents());
    }

    #[test]
    fn numbers_outside_their_range_are_rejected() {
        let file = TestFile::new("range", None);
        let mut document = ConfigDocument::load(file.path()).unwrap();
        assert!(document.set("animations.duration-scale", "-1").is_err());
        assert!(document.set("render.exposure.adaptation.light-to-dark", "11").is_err());
        document.set("render.exposure.adaptation.light-to-dark", "10").unwrap();
        assert_eq!(
            document
                .config()
                .render
                .exposure
                .adaptation
                .light_to_dark_half_life_seconds,
            10.0
        );
    }

    #[test]
    fn toggling_cycles_choices_and_flips_switches() {
        let file = TestFile::new("toggle", None);
        let mut document = ConfigDocument::load(file.path()).unwrap();
        document.toggle("render.tonemapper").unwrap();
        document.toggle("render.stars").unwrap();
        let config = document.config();
        assert_eq!(config.render.tonemapper, Tonemapper::AgxPunchy);
        assert!(!config.render.stars);
        assert!(document.toggle("render.bloom.strength").is_err());
    }

    #[test]
    fn a_prefix_lists_every_key_below_it() {
        let file = TestFile::new("prefix", None);
        let document = ConfigDocument::load(file.path()).unwrap();
        let listing = document.get("render.exposure").unwrap();
        assert!(listing.contains("render.exposure.mode eye-adaptation"), "{listing}");
        assert!(
            listing.contains("render.exposure.adaptation.light-to-dark 1.5"),
            "{listing}"
        );
        assert!(document.get("render.nothing").is_err());
    }
}
