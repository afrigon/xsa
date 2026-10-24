use crate::input::InputButton;

#[derive(Clone, Debug, PartialEq)]
pub struct ButtonPress {
    pub button: InputButton,
    pub held: Vec<InputButton>,
}
