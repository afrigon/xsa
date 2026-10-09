/// The error a pass returns from `RenderPass::record`.
pub type PassError = Box<dyn std::error::Error + Send + Sync>;
