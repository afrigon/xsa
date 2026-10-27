use usage::Cli;

use super::Command;

#[derive(Cli)]
#[usage(bin = "", unknown_flags = "error", args_override_self = false)]
pub struct CommandLine {
    #[usage(subcommand)]
    pub command: Command,
}

impl CommandLine {
    // The command `words` name, or why they name none, in plain text.
    pub fn parse_words(words: &[String]) -> Result<Command, String> {
        let arguments: Vec<&std::ffi::OsStr> = words.iter().map(std::ffi::OsStr::new).collect();

        CommandLine::parse_from(&arguments)
            .map(|line| line.command)
            .map_err(|err| usage::render_failure_plain(CommandLine::spec(), &arguments, &err))
    }
}
