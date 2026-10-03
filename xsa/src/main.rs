#[cfg(not(feature = "server"))]
compile_error!("xsa needs the `server` feature; `client` enables it too");

use std::net::IpAddr;
use std::path::PathBuf;

use anyhow::Context;
use usage::complete::Shell;
use usage::spec::{Candidate, CompleteCtx};
use usage::{Args, Cli, Subcommands};
use xsa_commands::{completion, ipc};
use xsa_server::WorldOptions;
use xsa_server::dedicated::{self, DedicatedOptions};

#[derive(Cli)]
#[usage(
    bin = "xsa",
    version,
    about = "xsa space flight simulator",
    completion,
    unknown_flags = "error",
    args_override_self = false
)]
#[cfg_attr(
    feature = "client",
    usage(default_subcommand = "client", default_subcommand_flags, default_subcommand_on_empty)
)]
struct Arguments {
    #[usage(subcommand)]
    command: Command,
}

#[derive(Subcommands)]
enum Command {
    #[cfg(feature = "client")]
    /// Run the game, with an integrated server unless --remote is given
    Client(ClientArguments),
    /// Run a dedicated server
    Server(ServerArguments),
    /// Send a command to a running xsa instance, e.g. xsa ipc time rate 100
    Ipc(IpcArguments),
    /// Print the shell completion script, e.g. xsa completion fish | source
    Completion(CompletionArguments),
}

#[derive(Args)]
struct CompletionArguments {
    #[usage(arg, choices("bash", "elvish", "fish", "nu", "powershell", "zsh"))]
    shell: String,
}

#[derive(Args)]
struct IpcArguments {
    #[usage(
        long,
        env = "XSA_INSTANCE",
        complete = complete_instances,
        help = "Instance to send to; required when several are running"
    )]
    instance: Option<String>,
    #[usage(long, help = "List the running instances")]
    list: bool,
    #[usage(
        arg,
        trailing_var_arg,
        allow_hyphen_values,
        complete = complete_ipc_words,
        help = "The command, as typed at the prompt"
    )]
    words: Vec<String>,
}

#[derive(Args)]
struct WorldArguments {
    #[usage(long, default = "packs", help = "Directory containing the installed packs")]
    packs_directory: PathBuf,
    #[usage(
        long,
        delimiter = ',',
        default = "base",
        default = "system-solar",
        help = "Packs to load, in stack order (later packs override earlier ones)"
    )]
    packs: Vec<String>,
    #[usage(
        long,
        help = "Simulation to run, e.g. system-solar:sol; defaults to the only one installed"
    )]
    simulation: Option<String>,
}

#[cfg(feature = "client")]
#[derive(Args)]
struct ClientArguments {
    #[usage(long, help = "Name for xsa ipc --instance; defaults to client-<pid>")]
    instance: Option<String>,
    #[usage(
        long,
        env_fallback("USER", "USERNAME"),
        default = "player",
        help = "Player name shown to other players"
    )]
    player: String,
    #[usage(
        long,
        requires("--fingerprint"),
        help = "Connect to a remote server at <host>:<port>; IPv6 hosts go in brackets, e.g. [::1]:1969"
    )]
    remote: Option<String>,
    #[usage(
        long,
        requires("--remote"),
        help = "Certificate fingerprint the remote server printed at startup"
    )]
    fingerprint: Option<String>,
    #[usage(flatten)]
    world: WorldArguments,
}

#[derive(Args)]
struct ServerArguments {
    #[usage(long, help = "Name for xsa ipc --instance; defaults to server-<pid>")]
    instance: Option<String>,
    #[usage(long, default = "::1", help = "Address to listen on")]
    host: IpAddr,
    #[usage(long, default = "1969", help = "UDP port to listen on")]
    port: u16,
    #[usage(long, default = ".xsa/server", help = "Directory holding the server identity")]
    identity: PathBuf,
    #[usage(flatten)]
    world: WorldArguments,
}

impl WorldArguments {
    fn into_options(self) -> WorldOptions {
        WorldOptions {
            packs_directory: self.packs_directory,
            packs: self.packs,
            simulation: self.simulation,
        }
    }
}

fn main() -> anyhow::Result<()> {
    match Arguments::parse().command {
        #[cfg(feature = "client")]
        Command::Client(arguments) => run_client(arguments),
        Command::Server(arguments) => dedicated::run(DedicatedOptions {
            instance: arguments.instance,
            host: arguments.host,
            port: arguments.port,
            identity: arguments.identity,
            world: arguments.world.into_options(),
        }),
        Command::Ipc(arguments) => run_ipc(arguments),
        Command::Completion(arguments) => {
            let shell = Shell::from_name(&arguments.shell).context("unsupported shell")?;
            print!("{}", Arguments::completion_script(shell));
            Ok(())
        }
    }
}

fn complete_instances<Partial>(_partial: &Partial, _context: &CompleteCtx<'_>) -> Vec<Candidate<'static>> {
    ipc::list()
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
    let completions = ipc::complete(instance.as_deref(), line.clone(), line.len())
        .unwrap_or_else(|_| completion::complete(&line, line.len(), |_| Vec::new()));
    completions
        .candidates
        .into_iter()
        .map(|candidate| match candidate.description {
            Some(description) => Candidate::described(candidate.value, description),
            None => Candidate::new(candidate.value),
        })
        .collect()
}

fn run_ipc(arguments: IpcArguments) -> anyhow::Result<()> {
    if arguments.list {
        for instance in ipc::list()? {
            println!("{instance}");
        }
        return Ok(());
    }
    anyhow::ensure!(!arguments.words.is_empty(), "give a command to send, or --list");
    let output = ipc::send(arguments.instance.as_deref(), arguments.words)?;
    println!("{}", output.text.trim_end());
    if !output.succeeded {
        std::process::exit(1);
    }
    Ok(())
}

#[cfg(feature = "client")]
fn run_client(arguments: ClientArguments) -> anyhow::Result<()> {
    let remote = match arguments.remote {
        Some(address) => Some(xsa_client::RemoteServer {
            address,
            fingerprint: arguments.fingerprint.context("--remote requires --fingerprint")?,
        }),
        None => None,
    };
    xsa_client::run(xsa_client::ClientOptions {
        instance: arguments.instance,
        player: arguments.player,
        remote,
        world: arguments.world.into_options(),
    })
}
