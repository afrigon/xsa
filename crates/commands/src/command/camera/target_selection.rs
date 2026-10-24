use xsa_core::simulation::BodyCategory;

// What `camera target` orbits: a body named by id, or one found from the body the camera is about.
#[derive(Clone, Debug, PartialEq)]
pub enum TargetSelection {
    Body { id: String },
    Next { category: Option<BodyCategory> },
    Previous { category: Option<BodyCategory> },
    Parent,
}
