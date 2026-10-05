// Where a command typed by a player may run: anywhere, or only while the game is played, with no menu open.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Availability {
    Anywhere,
    InGame,
}
