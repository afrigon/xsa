use super::{Availability, Route};

pub trait Routable {
    fn route(self) -> Route;

    fn availability(&self) -> Availability;
}
