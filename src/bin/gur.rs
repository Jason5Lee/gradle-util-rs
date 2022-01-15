use clap::{Parser, Subcommand};
use gradle_util_rs::LoggedSideEffect;
use std::time::Duration;

#[derive(Parser)]
#[clap(name = "gur", version = "0.1.0", about = "A Gradle utility")]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[clap(about = "Watch for the new Gradle project and set the gradle version")]
    SetNew {
        #[clap(long, help = "The Gradle version for the new project")]
        gradle_version: String,
        #[clap(
            long,
            required = true,
            help = "Directories to be watched recursively for the new projects. You can have multiple watched directories."
        )]
        watch_dir: Vec<String>,
        #[clap(long, default_value = "1s", parse(try_from_str = humantime::parse_duration), help = "Duration of file watching delay. Default to 1 second.")]
        watch_duration: Duration,
    },
    #[clap(about = "Update the Gradle version if it matches the major and the minor number")]
    Update {
        #[clap(
            long,
            required = true,
            help = "Version that may be updated to. You can have multiple version parameters."
        )]
        version: Vec<String>,
    },
}

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    log::debug!("DEBUG ON");

    let cli = Cli::parse();

    match cli.command {
        Command::SetNew {
            gradle_version,
            watch_dir,
            watch_duration,
        } => gradle_util_rs::set_new::set_new(watch_dir, gradle_version, watch_duration),
        Command::Update { version: versions } => gradle_util_rs::update::update(versions),
    }
    .ignore_logged_error()
}
