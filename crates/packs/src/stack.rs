use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};

use crate::{Document, Id, Manifest, PackData, ParseContext};

pub const DOCUMENT_EXTENSION: &str = "kdl";

const DATA_DIRECTORY: &str = "data";
const RESOURCES_DIRECTORY: &str = "resources";
const TEXTURES: &str = "textures";
const TEXTURE_EXTENSION: &str = "dds";

pub struct PackStack {
    manifests: Vec<Manifest>,
    files: HashMap<String, PathBuf>,
}

impl PackStack {
    pub fn load(directory: &Path, pack_ids: &[String]) -> anyhow::Result<PackStack> {
        let mut manifests: Vec<Manifest> = Vec::new();
        let mut files = HashMap::new();

        for pack_id in pack_ids {
            let pack_directory = directory.join(pack_id);
            let manifest = Manifest::read(&pack_directory, pack_id)?;

            for dependency in &manifest.dependencies {
                let Some(loaded) = manifests.iter().find(|loaded| loaded.id == dependency.pack) else {
                    bail!(
                        "pack {pack_id} depends on {}, which must be listed before it",
                        dependency.pack
                    );
                };

                if !dependency.requirement.matches(&loaded.version) {
                    bail!(
                        "pack {pack_id} needs {} {}, but {} is loaded",
                        dependency.pack,
                        dependency.requirement,
                        loaded.version
                    );
                }
            }

            PackStack::index_pack(&pack_directory, &mut files)?;
            manifests.push(manifest);
        }

        Ok(PackStack { manifests, files })
    }

    pub fn manifests(&self) -> &[Manifest] {
        &self.manifests
    }

    pub fn load_data<T: PackData>(&self, id: &Id) -> anyhow::Result<T> {
        let path = self.data(T::KIND, id)?;
        let document = Document::read(path)?;
        let context = ParseContext { stack: self, id };

        T::parse(&document, &context).with_context(|| format!("{}", path.display()))
    }

    pub fn data(&self, kind: &str, id: &Id) -> anyhow::Result<&Path> {
        self.find(DATA_DIRECTORY, kind, id, DOCUMENT_EXTENSION)
    }

    pub fn resource(&self, kind: &str, id: &Id, extension: &str) -> anyhow::Result<&Path> {
        self.find(RESOURCES_DIRECTORY, kind, id, extension)
    }

    pub fn texture(&self, id: &Id) -> anyhow::Result<&Path> {
        self.resource(TEXTURES, id, TEXTURE_EXTENSION)
    }

    pub fn data_ids(&self, kind: &str) -> Vec<Id> {
        let suffix = format!(".{DOCUMENT_EXTENSION}");
        let mut ids: Vec<Id> = self
            .files
            .keys()
            .filter_map(|key| {
                let rest = key.strip_prefix(DATA_DIRECTORY)?.strip_prefix('/')?;
                let (namespace, rest) = rest.split_once('/')?;
                let path = rest.strip_prefix(kind)?.strip_prefix('/')?.strip_suffix(&suffix)?;
                Some(Id {
                    namespace: namespace.to_string(),
                    path: path.to_string(),
                })
            })
            .collect();
        ids.sort_by_key(Id::to_string);
        ids
    }

    fn find(&self, section: &str, kind: &str, id: &Id, extension: &str) -> anyhow::Result<&Path> {
        let key = format!("{section}/{}/{kind}/{}.{extension}", id.namespace, id.path);
        self.files
            .get(&key)
            .map(PathBuf::as_path)
            .with_context(|| format!("no loaded pack provides {kind} {id} ({key})"))
    }

    fn index_pack(pack_directory: &Path, files: &mut HashMap<String, PathBuf>) -> anyhow::Result<()> {
        for entry in fs::read_dir(pack_directory).with_context(|| format!("reading {}", pack_directory.display()))? {
            let namespace_directory = entry?.path();

            if !namespace_directory.is_dir() {
                continue;
            }

            let namespace = namespace_directory
                .file_name()
                .expect("directory entries have a name")
                .to_string_lossy()
                .into_owned();

            for section in [DATA_DIRECTORY, RESOURCES_DIRECTORY] {
                let section_directory = namespace_directory.join(section);
                PackStack::index_directory(
                    &section_directory,
                    &section_directory,
                    &format!("{section}/{namespace}"),
                    files,
                )?;
            }
        }

        Ok(())
    }

    fn index_directory(
        section_directory: &Path,
        directory: &Path,
        key_prefix: &str,
        files: &mut HashMap<String, PathBuf>,
    ) -> anyhow::Result<()> {
        if !directory.is_dir() {
            return Ok(());
        }

        for entry in fs::read_dir(directory).with_context(|| format!("reading {}", directory.display()))? {
            let path = entry?.path();

            if path.is_dir() {
                PackStack::index_directory(section_directory, &path, key_prefix, files)?;
                continue;
            }

            let relative = path
                .strip_prefix(section_directory)
                .expect("indexed files are inside their section");
            let relative = relative
                .components()
                .map(|component| component.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            files.insert(format!("{key_prefix}/{relative}"), path);
        }

        Ok(())
    }
}
