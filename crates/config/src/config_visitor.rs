use crate::{ConfigSection, ConfigValue, NumberRange};

pub trait ConfigVisitor: Sized {
    fn enter(&mut self, name: &str);

    fn leave(&mut self);

    fn checked<T: ConfigValue>(&mut self, name: &str, value: &mut T, check: &dyn Fn(&T) -> anyhow::Result<()>);

    fn group(&mut self, name: &str, visit: impl FnOnce(&mut Self)) {
        self.enter(name);
        visit(self);
        self.leave();
    }

    fn section(&mut self, name: &str, section: &mut impl ConfigSection) {
        self.group(name, |visitor| section.visit(visitor));
    }

    fn value<T: ConfigValue>(&mut self, name: &str, value: &mut T) {
        self.checked(name, value, &|_| Ok(()));
    }

    fn number(&mut self, name: &str, value: &mut f32, range: NumberRange) {
        self.checked(name, value, &|number| range.check(*number));
    }
}
