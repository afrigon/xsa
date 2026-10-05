use std::cell::Cell;
use std::rc::Rc;

// How much the main menu's presentation shows: 1 frames the spawn body for the menu, 0 leaves the camera to the game.
// Menus are counted because a main menu replacing another loads before the old one unloads.
#[derive(Clone, Default)]
pub struct MenuPresence {
    weight: Rc<Cell<f64>>,
    menus: Rc<Cell<usize>>,
}

impl MenuPresence {
    pub fn weight(&self) -> f64 {
        self.weight.get()
    }

    pub fn set_weight(&self, weight: f64) {
        self.weight.set(weight);
    }

    pub fn menu_loaded(&self) {
        self.menus.set(self.menus.get() + 1);
        self.weight.set(1.0);
    }

    pub fn menu_unloaded(&self) {
        let menus = self.menus.get() - 1;
        self.menus.set(menus);

        if menus == 0 {
            self.weight.set(0.0);
        }
    }
}
