use resource_loader::terrain::HeightRange;

/// A page's data in its page format and, for page heights, the range its values span.
pub struct EncodedPage {
    pub data: Vec<u8>,
    pub range: Option<HeightRange>,
}
