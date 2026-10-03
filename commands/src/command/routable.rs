use super::Route;

pub trait Routable {
    fn route(self) -> Route;
}
