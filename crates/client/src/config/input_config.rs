use xsa_config::{ConfigSection, ConfigVisitor};
use xsa_input::{InputEvent, InputSource};

use super::{BindConfig, InputSourceConfig};

#[derive(Clone, Debug, PartialEq)]
pub struct InputConfig {
    pub enabled: bool,
    pub raw: InputSourceConfig,
    pub commands: InputSourceConfig,
    pub bind: BindConfig,
}

impl InputConfig {
    pub fn accepts(&self, event: &InputEvent) -> bool {
        event.kind.ends_input() || self.accepts_source(event.source)
    }

    pub fn accepts_source(&self, source: InputSource) -> bool {
        self.enabled && self.source(source).enabled
    }

    pub fn disabled_sources(&self) -> Vec<InputSource> {
        InputSource::ALL
            .into_iter()
            .filter(|source| !self.accepts_source(*source))
            .collect()
    }

    fn source(&self, source: InputSource) -> &InputSourceConfig {
        match source {
            InputSource::Raw => &self.raw,
            InputSource::Command => &self.commands,
        }
    }
}

impl ConfigSection for InputConfig {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        visitor.value("enabled", &mut self.enabled);
        visitor.section("raw", &mut self.raw);
        visitor.section("commands", &mut self.commands);
        visitor.section("bind", &mut self.bind);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use xsa_input::{InputButton, InputEventKind, Key};

    fn event(source: InputSource, kind: InputEventKind) -> InputEvent {
        InputEvent { source, kind }
    }

    #[test]
    fn a_disabled_source_holds_back_presses_but_not_releases() {
        let mut input = Config::default().input;
        input.raw.enabled = false;
        let button = InputButton::Key(Key::W);

        assert!(!input.accepts(&event(InputSource::Raw, InputEventKind::Pressed { button })));
        assert!(input.accepts(&event(InputSource::Raw, InputEventKind::Released { button })));
        assert!(input.accepts(&event(InputSource::Command, InputEventKind::Pressed { button })));
        assert_eq!(input.disabled_sources(), vec![InputSource::Raw]);
    }

    #[test]
    fn disabling_input_disables_every_source() {
        let mut input = Config::default().input;
        input.enabled = false;

        assert_eq!(input.disabled_sources(), InputSource::ALL.to_vec());
    }
}
