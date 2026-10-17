use std::num::NonZeroUsize;

use kdl::KdlValue;
use resource_vulkan::ResourceLimits;

use super::{ConfigKey, ConfigLayer, ConfigValueKind, ConfigValues};
use crate::config::ByteSize;

pub struct ResourcesDocument {
    max_cpu_cores: Option<Option<u32>>,
    max_memory_usage: Option<Option<ByteSize>>,
    max_video_memory_usage: Option<Option<ByteSize>>,
}

impl ResourcesDocument {
    pub const MAX_CPU_CORES: ConfigKey = ConfigKey::session(
        "resources.max-cpu-cores",
        ConfigValueKind::Optional {
            value: &ConfigValueKind::Count,
        },
    );
    pub const MAX_MEMORY_USAGE: ConfigKey = ConfigKey::session(
        "resources.max-memory-usage",
        ConfigValueKind::Optional {
            value: &ConfigValueKind::ByteSize,
        },
    );
    pub const MAX_VIDEO_MEMORY_USAGE: ConfigKey = ConfigKey::session(
        "resources.max-video-memory-usage",
        ConfigValueKind::Optional {
            value: &ConfigValueKind::ByteSize,
        },
    );

    pub fn read(values: &ConfigValues) -> ResourcesDocument {
        ResourcesDocument {
            max_cpu_cores: values.optional_count(ResourcesDocument::MAX_CPU_CORES),
            max_memory_usage: values.optional_byte_size(ResourcesDocument::MAX_MEMORY_USAGE),
            max_video_memory_usage: values.optional_byte_size(ResourcesDocument::MAX_VIDEO_MEMORY_USAGE),
        }
    }

    pub fn into_config(self, defaults: &ResourceLimits) -> ResourceLimits {
        let bytes = |size: Option<ByteSize>| size.map(|size| size.bytes);

        ResourceLimits {
            max_cpu_cores: self.max_cpu_cores.map_or(defaults.max_cpu_cores, |cores| {
                cores.and_then(|cores| NonZeroUsize::new(cores as usize))
            }),
            max_memory_usage: self.max_memory_usage.map_or(defaults.max_memory_usage, bytes),
            max_video_memory_usage: self
                .max_video_memory_usage
                .map_or(defaults.max_video_memory_usage, bytes),
        }
    }

    pub fn write(config: &ResourceLimits, layer: &mut ConfigLayer) {
        let size = |bytes: Option<u64>| {
            bytes.map_or(KdlValue::Null, |bytes| {
                KdlValue::String(ByteSize { bytes }.config_name())
            })
        };
        layer.set(
            ResourcesDocument::MAX_CPU_CORES.path,
            config
                .max_cpu_cores
                .map_or(KdlValue::Null, |cores| KdlValue::Integer(cores.get() as i128)),
        );
        layer.set(ResourcesDocument::MAX_MEMORY_USAGE.path, size(config.max_memory_usage));
        layer.set(
            ResourcesDocument::MAX_VIDEO_MEMORY_USAGE.path,
            size(config.max_video_memory_usage),
        );
    }
}
