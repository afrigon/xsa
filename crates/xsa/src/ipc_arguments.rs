use usage::Args;
use usage::spec::{Candidate, CompleteCtx};
use xsa_commands::completion::Completions;
use xsa_commands::ipc::IpcClient;

#[derive(Args)]
pub struct IpcArguments {
    #[usage(
        long,
        env = "XSA_INSTANCE",
        complete = complete_instances,
        help = "Instance to send to; required when several are running"
    )]
    pub instance: Option<String>,
    #[usage(long, help = "List the running instances")]
    pub list: bool,
    #[usage(
        arg,
        trailing_var_arg,
        allow_hyphen_values,
        complete = complete_ipc_words,
        help = "The command, as typed at the prompt"
    )]
    pub words: Vec<String>,
}

impl IpcArguments {
    pub fn run(self) -> anyhow::Result<()> {
        if self.list {
            for instance in IpcClient::list()? {
                println!("{instance}");
            }

            return Ok(());
        }

        anyhow::ensure!(!self.words.is_empty(), "give a command to send, or --list");
        let output = IpcClient::new(self.instance).send(self.words)?;
        println!("{}", output.text.trim_end());

        if !output.succeeded {
            std::process::exit(1);
        }

        Ok(())
    }
}

// usage-rs completers are plain functions.
fn complete_instances<Partial>(_partial: &Partial, _context: &CompleteCtx<'_>) -> Vec<Candidate<'static>> {
    IpcClient::list()
        .unwrap_or_default()
        .into_iter()
        .map(Candidate::new)
        .collect()
}

// The words after `ipc` belong to the instance's command tree, so the instance completes them.
fn complete_ipc_words<Partial>(_partial: &Partial, context: &CompleteCtx<'_>) -> Vec<Candidate<'static>> {
    let mut instance = std::env::var("XSA_INSTANCE").ok();
    let mut words = Vec::new();
    let mut given = context.command_words.iter();

    while let Some(word) = given.next() {
        match word.as_str() {
            "--instance" if words.is_empty() => instance = given.next().cloned(),
            "--list" if words.is_empty() => {}
            word => words.push(word.to_string()),
        }
    }

    let mut line = shlex::try_join(words.iter().map(String::as_str)).unwrap_or_default();

    if !line.is_empty() {
        line.push(' ');
    }

    line.push_str(context.prefix);
    let completions = IpcClient::new(instance)
        .complete(line.clone(), line.len())
        .unwrap_or_else(|_| Completions::compute(&line, line.len(), |_| Vec::new()));
    completions
        .candidates
        .into_iter()
        .map(|candidate| {
            let completion = match candidate.description {
                Some(description) => Candidate::described(candidate.value, description),
                None => Candidate::new(candidate.value),
            };

            match candidate.display {
                Some(display) => completion.displayed(display),
                None => completion,
            }
        })
        .collect()
}
