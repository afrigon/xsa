// A full-page screen of the interface, as `ui push` and `ui set` name it.
#[derive(usage::ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Screen {
    MainMenu,
    Config,
    Load,
    Game,
    PauseMenu,
}
