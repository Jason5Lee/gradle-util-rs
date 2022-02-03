use clap::{Parser, Subcommand};
use gradle_util_rs::LoggedSideEffect;
use std::time::Duration;

#[derive(Parser)]
#[clap(name = "gur", version = "0.1.0-alpha.2", about = "A Gradle utility")]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[clap(about = "Watch for the new Gradle project and set the gradle version")]
    SetNew {
        #[clap(help = "The gradle wrapper version to be set for the new projects.")]
        version: String,
        #[clap(
            long,
            required = true,
            help = "Directories to be watched recursively for the new projects. You can have multiple watched directories."
        )]
        watch_dir: Vec<String>,
        #[clap(long, default_value = "1s", parse(try_from_str = humantime::parse_duration), help = "Duration of file watching delay. Default to 1 second.")]
        watch_duration: Duration,
    },

    #[clap(about = "Change the gradle wrapper version")]
    Chver {
        #[clap(help = "The version you want to change to.")]
        version: String,
        #[clap(long, help = "Enable the yolo mode. It will change the gradle-wrapper.properties file before running the wrapper task. With this flag, the gradle distribution of the old version won't be downloaded. But it may have potential problems.")]
        yolo: bool,
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
            version,
            watch_dir,
            watch_duration,
        } => gradle_util_rs::set_new::set_new(watch_dir, version, watch_duration),
        Command::Chver { version, yolo } => gradle_util_rs::chver::chver(version, yolo),
    }
    .ignore_logged_error()
}
