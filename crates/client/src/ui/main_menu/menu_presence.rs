use std::cell::Cell;
use std::rc::Rc;

// How much the main menu's presentation shows: 1 frames the spawn body for the menu, 0 leaves the camera to the game.
#[derive(Clone, Default)]
pub struct MenuPresence {
    weight: Rc<Cell<f64>>,
}

impl MenuPresence {
    pub fn weight(&self) -> f64 {
        self.weight.get()
    }

    pub fn set_weight(&self, weight: f64) {
        self.weight.set(weight);
    }
}
