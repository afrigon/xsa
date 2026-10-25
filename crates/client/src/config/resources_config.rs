use std::num::NonZeroUsize;
use std::time::Duration;

use resource_vulkan::ResourceLimits;
use xsa_config::{ByteSize, ConfigSection, ConfigVisitor};

// The limits of the service that loads the files content refers to.
#[derive(Clone, Debug, PartialEq)]
pub struct ResourcesConfig {
    pub memory_max_usage: Option<ByteSize>,
    pub video_memory_max_usage: Option<ByteSize>,
    pub workers_max_count: Option<NonZeroUsize>,
    pub read_delay: Option<Duration>,
}

impl ResourcesConfig {
    pub fn from_limits(limits: ResourceLimits) -> ResourcesConfig {
        let size = |bytes: Option<u64>| bytes.map(|bytes| ByteSize { bytes });

        ResourcesConfig {
            memory_max_usage: size(limits.max_memory_usage),
            video_memory_max_usage: size(limits.max_video_memory_usage),
            workers_max_count: limits.max_cpu_cores,
            read_delay: limits.read_delay,
        }
    }

    pub fn limits(&self) -> ResourceLimits {
        let bytes = |size: Option<ByteSize>| size.map(|size| size.bytes);

        ResourceLimits {
            max_cpu_cores: self.workers_max_count,
            max_memory_usage: bytes(self.memory_max_usage),
            max_video_memory_usage: bytes(self.video_memory_max_usage),
            read_delay: self.read_delay,
        }
    }
}

impl ConfigSection for ResourcesConfig {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        visitor.group("memory", |visitor| {
            visitor.value("max-usage", &mut self.memory_max_usage)
        });
        visitor.group("video-memory", |visitor| {
            visitor.value("max-usage", &mut self.video_memory_max_usage)
        });
        visitor.group("workers", |visitor| {
            visitor.value("max-count", &mut self.workers_max_count)
        });
        visitor.value("read-delay", &mut self.read_delay);
    }
}
