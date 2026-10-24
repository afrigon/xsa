use super::{Anchor, CameraPose, Controls, ControlsContext, DebugControls, OrbitControls};
use crate::config::InputConfig;
use crate::input::Input;

pub enum AnyControls {
    Orbit(OrbitControls),
    Debug(DebugControls),
}

impl AnyControls {
    fn controls(&self) -> &dyn Controls {
        match self {
            AnyControls::Orbit(controls) => controls,
            AnyControls::Debug(controls) => controls,
        }
    }

    fn controls_mut(&mut self) -> &mut dyn Controls {
        match self {
            AnyControls::Orbit(controls) => controls,
            AnyControls::Debug(controls) => controls,
        }
    }
}

impl Controls for AnyControls {
    fn update(&mut self, context: &ControlsContext) -> CameraPose {
        self.controls_mut().update(context)
    }

    fn anchor(&self) -> Anchor {
        self.controls().anchor()
    }

    fn wants_mouse_capture(&self, input: &Input) -> Option<bool> {
        self.controls().wants_mouse_capture(input)
    }

    fn name(&self) -> &'static str {
        self.controls().name()
    }

    fn speed(&self, config: &InputConfig) -> Option<f64> {
        self.controls().speed(config)
    }
}
