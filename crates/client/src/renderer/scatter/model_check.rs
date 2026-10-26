// Whether a model's mesh nodes match its materials one for one, known once its mesh loads.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(in crate::renderer) enum ModelCheck {
    Pending,
    Matched,
    Mismatched,
}
