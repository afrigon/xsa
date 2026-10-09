use crate::ResourceUsage;

#[derive(Clone, Debug)]
pub struct ImageUsageDescription {
    pub image: &'static str,
    pub level: Option<u32>,
    pub usage: ResourceUsage,
}
