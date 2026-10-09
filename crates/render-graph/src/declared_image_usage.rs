use crate::ResourceUsage;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DeclaredImageUsage {
    pub pass: usize,
    pub step: u32,
    pub image: usize,
    pub level: Option<u32>,
    pub usage: ResourceUsage,
}
