use clap::{Args, Parser, Subcommand};
use gradle_util_rs::LoggedSideEffect;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser)]
#[clap(name = "gur", version = "0.1.0-alpha.6", about = "A Gradle utility")]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[clap(about = "Watch for the new Gradle project and set the gradle version")]
    SetNew {
        #[clap(
            required = true,
            help = "The gradle wrapper version to be set for the new projects."
        )]
        version: String,
        #[clap(
            required = true,
            parse(from_os_str),
            help = "Directories to be watched recursively for the new projects. You can have multiple watched directories."
        )]
        watch_dir: Vec<PathBuf>,
        #[clap(long, default_value = "1s", parse(try_from_str = humantime::parse_duration), help = "Duration of file watching delay. Default to 1 second.")]
        watch_duration: Duration,
    },

    #[clap(about = "Change the gradle wrapper version")]
    Chver {
        #[clap(help = "The version you want to change to.")]
        version: String,
        #[clap(
            long,
            short,
            parse(from_os_str),
            default_value = ".",
            help = "Project directory."
        )]
        project_dir: PathBuf,
        #[clap(
            long,
            help = "Enable the yolo mode. It will change the gradle-wrapper.properties file before running the wrapper task. With this flag, the gradle distribution of the old version won't be downloaded. But it may not work as expected."
        )]
        yolo: bool,
    },

    #[clap(about = "Create project from the template")]
    Template(Template),
}
#[derive(Debug, Args)]
struct Template {
    #[clap(subcommand)]
    command: TemplateCommands,
}
#[derive(Debug, Subcommand)]
enum TemplateCommands {
    #[clap(about = "List the template information")]
    List {
        #[clap(help = "The template name. List all templates if omitted.")]
        name: Option<String>,
    },
    #[clap(about = "Create a new project from a template")]
    New {
        #[clap(required = true, help = "The template name")]
        name: String,
        #[clap(short, long, help = "Use iterative mode")]
        iterative: bool,
        #[clap(short, long, parse(from_os_str), help = "The output directory")]
        output: Option<PathBuf>,
        #[clap(short = 'D', parse(try_from_str = parse_key_val), multiple_occurrences(true), help = "Define the template arguments, e.g. -Dname=value")]
        defines: Vec<(String, String)>,
        #[clap(long, help = "Allow output directory to exist")]
        allow_exists: bool,
        #[clap(
            long,
            help = "Overwrite existing file, only useful with --allow-exists"
        )]
        overwrite: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::SetNew {
            version,
            watch_dir,
            watch_duration,
        } => gradle_util_rs::set_new::set_new(watch_dir, version, watch_duration),
        Command::Chver {
            version,
            project_dir,
            yolo,
        } => gradle_util_rs::chver::chver(project_dir, version, yolo),
        Command::Template(Template { command }) => match command {
            TemplateCommands::List { name } => gradle_util_rs::templates::list(name),
            TemplateCommands::New {
                name,
                iterative,
                output,
                defines,
                allow_exists,
                overwrite,
            } => gradle_util_rs::templates::new(
                name,
                output,
                defines,
                allow_exists,
                overwrite,
                iterative,
            ),
        },
    }
    .ignore_logged_error()
}

fn parse_key_val<T, U>(
    s: &str,
) -> Result<(T, U), Box<dyn std::error::Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: std::error::Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}
