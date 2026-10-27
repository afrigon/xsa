// Patches of a model across the ground: noise features about `size` meters across scale its weight, from 0 to its
// full value; a `contrast` above 1 sharpens their edges, leaving ground between them where it reaches 0.
#[derive(Clone, Copy, Debug)]
pub struct WeightNoise {
    pub size: f64,
    pub contrast: f64,
}
