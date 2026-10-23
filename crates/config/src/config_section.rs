use crate::ConfigVisitor;

pub trait ConfigSection {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V);
}
