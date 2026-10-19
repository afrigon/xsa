use std::ffi::{CStr, c_void};
use std::mem::ManuallyDrop;
use std::sync::Mutex;

use ash::{ext, vk};
use gpu_allocator::MemoryLocation;

use crate::{BindlessTable, Buffer, DeviceObject, Image, MemoryContext, MemoryError, Timeline, TransferQueue};

const VALIDATION_LAYER: &CStr = c"VK_LAYER_KHRONOS_validation";
// Messages from this injected layer are not ours.
const IGNORED_LAYER: &str = "obs-vkcapture";

/// A device without a surface, for tests that need a GPU. It enables what the crate's types require, uses a dedicated
/// transfer queue family when the GPU has one, and collects every validation warning and error when the validation
/// layer is installed.
pub struct TestDevice {
    // Dropped by hand, in this order, before the debug messenger and the instance.
    bindless: ManuallyDrop<BindlessTable>,
    transfer: ManuallyDrop<TransferQueue>,
    command_pool: ManuallyDrop<DeviceObject<vk::CommandPool>>,
    graphics: ManuallyDrop<Timeline>,
    memory: ManuallyDrop<MemoryContext>,
    debug_utils: Option<ext::debug_utils::Instance>,
    messenger: vk::DebugUtilsMessengerEXT,
    messages: Box<Mutex<Vec<String>>>,
    instance: ash::Instance,
    _entry: ash::Entry,
}

impl TestDevice {
    /// # Panics
    ///
    /// If no GPU supports what the crate requires.
    pub fn new() -> TestDevice {
        let entry = unsafe { ash::Entry::load() }.expect("the Vulkan loader is installed");
        let validation = unsafe { entry.enumerate_instance_layer_properties() }
            .expect("instance layers can be listed")
            .iter()
            .any(|layer| layer.layer_name_as_c_str() == Ok(VALIDATION_LAYER));
        let layers = if validation {
            vec![VALIDATION_LAYER.as_ptr()]
        } else {
            Vec::new()
        };
        let extensions = if validation {
            vec![ext::debug_utils::NAME.as_ptr()]
        } else {
            Vec::new()
        };
        let application = vk::ApplicationInfo::default().api_version(vk::API_VERSION_1_3);
        let instance_info = vk::InstanceCreateInfo::default()
            .application_info(&application)
            .enabled_layer_names(&layers)
            .enabled_extension_names(&extensions);
        let instance = unsafe { entry.create_instance(&instance_info, None) }.expect("an instance can be created");
        let messages = Box::new(Mutex::new(Vec::new()));
        let (debug_utils, messenger) = if validation {
            let debug_utils = ext::debug_utils::Instance::new(&entry, &instance);
            let messenger_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
                .message_severity(
                    vk::DebugUtilsMessageSeverityFlagsEXT::WARNING | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
                )
                .message_type(
                    vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                )
                .pfn_user_callback(Some(TestDevice::collect_message))
                .user_data(&*messages as *const Mutex<Vec<String>> as *mut c_void);
            let messenger = unsafe { debug_utils.create_debug_utils_messenger(&messenger_info, None) }
                .expect("a debug messenger can be created");
            (Some(debug_utils), messenger)
        } else {
            (None, vk::DebugUtilsMessengerEXT::null())
        };

        let DeviceChoice {
            physical_device,
            queue_family,
            transfer_family,
        } = TestDevice::pick_device(&instance);
        let priorities = [1.0];
        let mut queue_infos = vec![
            vk::DeviceQueueCreateInfo::default()
                .queue_family_index(queue_family)
                .queue_priorities(&priorities),
        ];

        if transfer_family != queue_family {
            queue_infos.push(
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(transfer_family)
                    .queue_priorities(&priorities),
            );
        }

        let device_extensions = [ext::memory_budget::NAME.as_ptr()];
        let features = vk::PhysicalDeviceFeatures::default().sampler_anisotropy(true);
        let mut vulkan_12_features = vk::PhysicalDeviceVulkan12Features::default()
            .buffer_device_address(true)
            .timeline_semaphore(true)
            .runtime_descriptor_array(true)
            .descriptor_binding_partially_bound(true)
            .descriptor_binding_sampled_image_update_after_bind(true)
            .descriptor_binding_storage_image_update_after_bind(true)
            .descriptor_binding_update_unused_while_pending(true);
        let mut vulkan_13_features = vk::PhysicalDeviceVulkan13Features::default().synchronization2(true);
        let device_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_infos)
            .enabled_extension_names(&device_extensions)
            .enabled_features(&features)
            .push_next(&mut vulkan_12_features)
            .push_next(&mut vulkan_13_features);
        let memory =
            MemoryContext::new(&instance, physical_device, &device_info).expect("a test device can be created");
        let device = memory.device();
        let queue = unsafe { device.get_device_queue(queue_family, 0) };
        let graphics = memory.create_timeline(queue).expect("a timeline can be created");
        let transfer_timeline = if transfer_family == queue_family {
            graphics.clone()
        } else {
            let transfer_queue = unsafe { device.get_device_queue(transfer_family, 0) };
            memory
                .create_timeline(transfer_queue)
                .expect("a timeline can be created")
        };
        let transfer = TransferQueue::new(&memory, transfer_timeline, transfer_family, queue_family)
            .expect("a transfer queue can be created");
        let bindless = BindlessTable::new(&memory, 1.0).expect("a bindless table can be created");
        let pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::TRANSIENT)
            .queue_family_index(queue_family);
        let command_pool = DeviceObject::new(
            &memory,
            unsafe { device.create_command_pool(&pool_info, None) }.expect("a command pool can be created"),
        );

        TestDevice {
            bindless: ManuallyDrop::new(bindless),
            transfer: ManuallyDrop::new(transfer),
            command_pool: ManuallyDrop::new(command_pool),
            graphics: ManuallyDrop::new(graphics),
            memory: ManuallyDrop::new(memory),
            debug_utils,
            messenger,
            messages,
            instance,
            _entry: entry,
        }
    }

    pub fn memory(&self) -> &MemoryContext {
        &self.memory
    }

    pub fn transfer(&self) -> &TransferQueue {
        &self.transfer
    }

    pub fn bindless(&self) -> &BindlessTable {
        &self.bindless
    }

    /// Submits an empty frame that uses every copy seen complete, as a frame of the game does, waits for it, and runs
    /// the deletions the GPU has reached.
    pub fn frame(&self) -> Result<(), MemoryError> {
        let command_buffer = self.command_buffer()?;
        let recording = self.graphics.begin(command_buffer)?;
        let point = recording.submit(&[self.transfer.visibility_wait()], &[])?;
        self.graphics.wait(point.value)?;
        self.free(command_buffer);

        self.memory.collect()
    }

    /// Copies layer `layer` of level 0 of `image`, in `layout`, back to the CPU: `size` bytes of tightly packed rows.
    pub fn read_layer(
        &self,
        image: &Image,
        layout: vk::ImageLayout,
        extent: vk::Extent2D,
        layer: u32,
        size: usize,
    ) -> Result<Vec<u8>, MemoryError> {
        let buffer = Buffer::new(
            &self.memory,
            "test readback",
            size as u64,
            vk::BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::GpuToCpu,
            &crate::QueueSharing::Exclusive,
        )?;
        let command_buffer = self.command_buffer()?;
        let recording = self.graphics.begin(command_buffer)?;
        let region = vk::BufferImageCopy::default()
            .image_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_array_layer(layer)
                    .layer_count(1),
            )
            .image_extent(extent.into());
        unsafe {
            self.memory.device().cmd_copy_image_to_buffer(
                recording.command_buffer(),
                image.handle(),
                layout,
                buffer.handle(),
                &[region],
            );
        }
        let point = recording.submit(&[self.transfer.visibility_wait()], &[])?;
        self.graphics.wait(point.value)?;
        self.free(command_buffer);
        let mut bytes = vec![0; size];
        buffer.read(&mut bytes)?;

        Ok(bytes)
    }

    /// The validation warnings and errors reported so far.
    pub fn validation_messages(&self) -> Vec<String> {
        self.messages
            .lock()
            .expect("the validation messages' lock is poisoned")
            .clone()
    }

    fn command_buffer(&self) -> Result<vk::CommandBuffer, MemoryError> {
        let allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.command_pool.handle())
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        Ok(unsafe { self.memory.device().allocate_command_buffers(&allocate_info) }?[0])
    }

    // Once the GPU finished it.
    fn free(&self, command_buffer: vk::CommandBuffer) {
        unsafe {
            self.memory
                .device()
                .free_command_buffers(self.command_pool.handle(), &[command_buffer])
        };
    }

    // The first GPU with a graphics queue, and its queue family dedicated to transfers if it has one.
    fn pick_device(instance: &ash::Instance) -> DeviceChoice {
        let devices = unsafe { instance.enumerate_physical_devices() }.expect("GPUs can be listed");

        devices
            .into_iter()
            .find_map(|device| {
                let families = unsafe { instance.get_physical_device_queue_family_properties(device) };
                let graphics = families
                    .iter()
                    .position(|family| family.queue_flags.contains(vk::QueueFlags::GRAPHICS))?;
                let transfer = families
                    .iter()
                    .position(|family| {
                        family.queue_flags.contains(vk::QueueFlags::TRANSFER)
                            && !family
                                .queue_flags
                                .intersects(vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE)
                    })
                    .unwrap_or(graphics);

                Some(DeviceChoice {
                    physical_device: device,
                    queue_family: graphics as u32,
                    transfer_family: transfer as u32,
                })
            })
            .expect("a GPU has a graphics queue")
    }

    unsafe extern "system" fn collect_message(
        _severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        _types: vk::DebugUtilsMessageTypeFlagsEXT,
        data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
        user_data: *mut c_void,
    ) -> vk::Bool32 {
        let messages = unsafe { &*(user_data as *const Mutex<Vec<String>>) };
        let message = unsafe { (*data).message_as_c_str() }
            .map(|message| message.to_string_lossy().into_owned())
            .unwrap_or_default();

        if !message.contains(IGNORED_LAYER)
            && let Ok(mut messages) = messages.lock()
        {
            messages.push(message);
        }

        vk::FALSE
    }
}

impl Drop for TestDevice {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.bindless);
            ManuallyDrop::drop(&mut self.transfer);
            ManuallyDrop::drop(&mut self.command_pool);
            ManuallyDrop::drop(&mut self.graphics);
            ManuallyDrop::drop(&mut self.memory);

            if let Some(debug_utils) = &self.debug_utils {
                debug_utils.destroy_debug_utils_messenger(self.messenger, None);
            }

            self.instance.destroy_instance(None);
        }
    }
}

// A GPU and the queue families a test device uses on it.
struct DeviceChoice {
    physical_device: vk::PhysicalDevice,
    queue_family: u32,
    transfer_family: u32,
}
