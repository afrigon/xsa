use std::rc::Rc;

// Read and write access to a value owned elsewhere, like SwiftUI's `@Binding`: a parent's state, or app data whose
// `set` decides what changing it means.
pub struct Binding<T: 'static> {
    get: Rc<dyn Fn() -> T>,
    set: Rc<dyn Fn(T)>,
}

impl<T: 'static> Binding<T> {
    pub fn new(get: impl Fn() -> T + 'static, set: impl Fn(T) + 'static) -> Binding<T> {
        Binding {
            get: Rc::new(get),
            set: Rc::new(set),
        }
    }

    pub fn get(&self) -> T {
        (self.get)()
    }

    pub fn set(&self, value: T) {
        (self.set)(value);
    }
}

impl<T: 'static> Clone for Binding<T> {
    fn clone(&self) -> Binding<T> {
        Binding {
            get: self.get.clone(),
            set: self.set.clone(),
        }
    }
}
