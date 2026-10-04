// A view drawn over the view controllers, shown or hidden on its own.
#[derive(usage::ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Overlay {
    DebugOverlay,
}
