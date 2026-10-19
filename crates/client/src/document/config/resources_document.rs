use std::num::NonZeroUsize;
use std::time::Duration;

use kdl::KdlValue;
use resource_vulkan::ResourceLimits;

use super::{ConfigKey, ConfigLayer, ConfigValueKind, ConfigValues, DurationName};
use crate::config::ByteSize;

pub struct ResourcesDocument {
    max_cpu_cores: Option<Option<u32>>,
    max_memory_usage: Option<Option<ByteSize>>,
    max_video_memory_usage: Option<Option<ByteSize>>,
    storage_debug_delay: Option<Option<Duration>>,
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
    pub const STORAGE_DEBUG_DELAY: ConfigKey = ConfigKey::session(
        "resources.storage.debug-delay",
        ConfigValueKind::Optional {
            value: &ConfigValueKind::Duration,
        },
    );

    pub fn read(values: &ConfigValues) -> ResourcesDocument {
        ResourcesDocument {
            max_cpu_cores: values.optional_count(ResourcesDocument::MAX_CPU_CORES),
            max_memory_usage: values.optional_byte_size(ResourcesDocument::MAX_MEMORY_USAGE),
            max_video_memory_usage: values.optional_byte_size(ResourcesDocument::MAX_VIDEO_MEMORY_USAGE),
            storage_debug_delay: values.optional_duration(ResourcesDocument::STORAGE_DEBUG_DELAY),
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
            read_delay: self.storage_debug_delay.unwrap_or(defaults.read_delay),
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
        layer.set(
            ResourcesDocument::STORAGE_DEBUG_DELAY.path,
            config
                .read_delay
                .map_or(KdlValue::Null, |delay| KdlValue::String(DurationName::write(delay))),
        );
    }
}
