use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use resource_vulkan::{LoadRequest, PageSource, ResourceGroup, ResourceHandle, Resources, StreamedSet};

use crate::resource_kinds::TerrainLayerFile;

// A terrain layer: its file's header loads first, then its pages stream through a set of the format it declares.
pub(in crate::renderer) struct StreamedLayer<S: PageSource> {
    name: String,
    file: ResourceHandle<TerrainLayerFile>,
    // Moved into the set when it is registered.
    source: Mutex<Option<S>>,
    set: OnceLock<StreamedSet<S>>,
}

impl<S: PageSource> StreamedLayer<S> {
    pub fn new(
        resources: &Resources,
        group: &mut ResourceGroup,
        path: PathBuf,
        source: S,
        name: String,
    ) -> StreamedLayer<S> {
        let file = resources.load(LoadRequest::new(path, name.clone()), ());
        group.add(&file);

        StreamedLayer {
            name,
            file,
            source: Mutex::new(Some(source)),
            set: OnceLock::new(),
        }
    }

    // Registers the set once the file's header is loaded; true when it did.
    pub fn update(&self, resources: &Resources) -> bool {
        if self.set.get().is_some() {
            return false;
        }

        let Some(file) = self.file.get() else {
            return false;
        };
        let Some(source) = self.source.lock().expect("a layer's source lock is poisoned").take() else {
            return false;
        };
        self.set
            .get_or_init(|| resources.stream(source, file.page_format(), self.name.clone()));

        true
    }

    // None until the file's header is loaded.
    pub fn set(&self) -> Option<&StreamedSet<S>> {
        self.set.get()
    }

    // None until it is loaded.
    pub fn file(&self) -> Option<&TerrainLayerFile> {
        self.file.get()
    }

    pub fn finest_level(&self) -> Option<u32> {
        self.set.get()?;
        self.file.get().map(TerrainLayerFile::finest_level)
    }
}
