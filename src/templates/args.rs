mod default_package;

use super::template_file::Args;
use crate::{log_error, Logged};
use fxhash::FxHashMap;
use std::io::{BufRead, Write};

const GROUP: &str = "group";
const GROUP_DESCRIPTION: &str = "Project group ID";
const ARTIFACT: &str = "artifact";
const ARTIFACT_DESCRIPTION: &str = "Project artifact ID";
const PACKAGE: &str = "package";
const PACKAGE_DESCRIPTION: &str = "Project root package";
const PACKAGE_DEFAULT_DESCRIPTION: &str =
    "If not specified, use <group>.<artifact> and try to fix the illegal part";
const VERSION: &str = "version";
const VERSION_DESCRIPTION: &str = "Project version";
const DEFAULT_VERSION: &str = "1.0.0-SNAPSHOT";
const TARGET_JVM: &str = "targetJvm";
const TARGET_JVM_DESCRIPTION: &str = "Target JVM version";

fn cli_read_arg(prompt: std::fmt::Arguments, default: Option<String>) -> std::io::Result<String> {
    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    let mut line = String::new();
    loop {
        write!(stdout, "{}", prompt)?;
        if let Some(d) = &default {
            write!(stdout, " ({d})")?;
        }
        write!(stdout, ":")?;
        stdout.flush()?;
        line.clear();
        stdin.read_line(&mut line)?;
        let value = line.trim();
        if value.is_empty() {
            if let Some(d) = default {
                return Ok(d);
            }
        } else {
            return Ok(value.to_string());
        }
    }
}
fn read_arg(
    arg: &str,
    description: &str,
    args: &mut FxHashMap<String, String>,
    default: Option<String>,
    iterative: bool,
) -> Result<(), Logged> {
    let arg_ref = args.entry(arg.to_string()).or_insert(String::new());

    let candidate_value = if arg_ref.is_empty() {
        default
    } else {
        Some(std::mem::take(arg_ref))
    };
    let value = if iterative {
        cli_read_arg(format_args!("Enter {description}"), candidate_value)
            .map_err(|err| log_error(format_args!("{err}")))
    } else {
        candidate_value.ok_or_else(|| log_error(format_args!("missing argument `{}`", arg)))
    };

    *arg_ref = value?;
    Ok(())
}
fn read_package(args: &mut FxHashMap<String, String>, iterative: bool) -> Result<(), Logged> {
    let candidate_value = if let Some(exists) = args.get_mut("package") {
        Ok(std::mem::take(exists))
    } else {
        default_package::get_package(&args["group"], &args["artifact"])
    };

    let value = if iterative {
        cli_read_arg(format_args!("Enter Package"), candidate_value.ok())
            .map_err(|err| log_error(format_args!("{err}")))
    } else {
        candidate_value.map_err(|err| log_error(format_args!("{err}")))
    };

    args.insert("package".to_string(), value?);
    Ok(())
}

pub(super) fn get_args(
    iterative: bool,
    cmdline_args_value: Vec<(String, String)>,
    args_info: Args,
) -> Result<FxHashMap<String, String>, Logged> {
    let mut result = FxHashMap::default();

    let Args { target_jvm, args } = args_info;

    for (k, v) in cmdline_args_value {
        if result.contains_key(&k) {
            return Err(log_error(format_args!("duplicate argument: {}", k)));
        }
        result.insert(k, v);
    }

    read_arg("group", "Group ID", &mut result, None, iterative)?;
    read_arg("artifact", "Artifact ID", &mut result, None, iterative)?;
    read_arg("version", "Version", &mut result, None, iterative)?;

    read_package(&mut result, iterative)?;
    result.insert(
        "packagePath".to_string(),
        result["package"].replace('.', "/"),
    );

    if let Some(target_jvm_info) = target_jvm {
        read_arg(
            "targetJvm",
            "Target JVM",
            &mut result,
            target_jvm_info.default,
            iterative,
        )?;
        result.insert(
            "targetJvmJava".to_string(),
            result["targetJvm"].replace('.', "_"),
        );
    }

    for (arg_name, arg_info) in args {
        read_arg(
            &arg_name,
            &arg_info.description,
            &mut result,
            arg_info.default,
            iterative,
        )?;
    }
    result.insert("dollar".to_string(), "$".to_string());
    Ok(result)
}

pub(super) fn print_args_list(args_info: Args) {
    println!(
        "    {GROUP}: {GROUP_DESCRIPTION}
    {ARTIFACT}: {ARTIFACT_DESCRIPTION}
    {PACKAGE}: {PACKAGE_DESCRIPTION}. {PACKAGE_DEFAULT_DESCRIPTION}
    {VERSION}: {VERSION_DESCRIPTION}, the default is {DEFAULT_VERSION}"
    );
    if let Some(target_jvm) = args_info.target_jvm {
        print!("    {TARGET_JVM}: {TARGET_JVM_DESCRIPTION}");
        if let Some(default) = target_jvm.default {
            print!(", the default is {}", default)
        }
        println!();
    }
    for (arg_name, arg_info) in args_info.args {
        print!("    {}: {}", arg_name, arg_info.description);
        if let Some(default) = arg_info.default {
            print!(", the default is {}", default);
        }
        println!();
    }
}
