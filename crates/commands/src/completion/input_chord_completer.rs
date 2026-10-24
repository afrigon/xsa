use usage::spec::{Candidate, CompleteCtx};
use xsa_config::ConfigValue;
use xsa_input::InputChord;

const CHORD_SEPARATOR: char = '+';

// Chords complete one button at a time; typing the separator moves on to the next one.
pub(crate) fn complete_input_chord<Partial>(_partial: &Partial, context: &CompleteCtx<'_>) -> Vec<Candidate<'static>> {
    let typed = context
        .prefix
        .rfind(CHORD_SEPARATOR)
        .map_or("", |index| &context.prefix[..=index]);

    InputChord::completions()
        .into_iter()
        .map(|button| Candidate::new(format!("{typed}{button}")).displayed(button))
        .collect()
}
