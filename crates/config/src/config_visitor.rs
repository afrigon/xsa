use crate::unconstrained::Unconstrained;
use crate::{ConfigConstraint, ConfigSection, ConfigValue, NumberRange};

pub trait ConfigVisitor: Sized {
    fn enter(&mut self, name: &str);

    fn leave(&mut self);

    fn constrained<T: ConfigValue>(&mut self, name: &str, value: &mut T, constraint: &dyn ConfigConstraint<T>);

    fn group(&mut self, name: &str, visit: impl FnOnce(&mut Self)) {
        self.enter(name);
        visit(self);
        self.leave();
    }

    fn section(&mut self, name: &str, section: &mut impl ConfigSection) {
        self.group(name, |visitor| section.visit(visitor));
    }

    fn value<T: ConfigValue>(&mut self, name: &str, value: &mut T) {
        self.constrained(name, value, &Unconstrained);
    }

    fn number(&mut self, name: &str, value: &mut f32, range: NumberRange) {
        self.constrained(name, value, &range);
    }
}
