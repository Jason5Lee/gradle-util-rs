mod default_package;

use std::collections::hash_map::Entry;
use std::io::Write;
use super::template_file::Args;
use crate::{log_error, Logged};
use fxhash::FxHashMap;

fn required(arg: &str) -> Result<String, Logged> {
    Err(log_error(format_args!("missing argument `{}`", arg)))
}

fn optional_default(value: Option<String>) -> impl FnOnce(&str) -> Result<String, Logged> {
    move |arg| match value { Some(v) => Ok(v), None => required(arg) }
}

fn read_arg(arg: &str, args: &mut FxHashMap<String, String>, default: impl FnOnce(&str) -> Result<String, Logged>, iterative: bool) -> Result<(), Logged> {
    let arg_ref = args.entry(arg.to_string()).or_insert(String::new());

    let candidate_value = if arg_ref.is_empty() {
        default(arg)
    } else {
        Ok(std::mem::take(arg_ref))
    };
    let value = if iterative {
        let mut stdout = std::io::stdout().lock();
        write!(stdout, "Enter {arg}").ok();
        if let Ok(default) = &candidate_value {
            write!(stdout, " ({default})").ok();
        }
        write!(stdout, ":").ok();
        stdout.flush().ok();
        drop(stdout);
        let mut user_input = String::new();
        std::io::stdin().read_line(&mut user_input)
            .map_err(|err| log_error(format_args!("{}", err)))?;
        if user_input.is_empty() {
            candidate_value
        } else {
            Ok(user_input.into())
        }
    } else {
        candidate_value
    };

    *arg_ref = value?;
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

    read_arg("group", &mut result, required, iterative)?;
    read_arg("artifact", &mut result, required, iterative)?;
    read_arg("version", &mut result, required, iterative)?;

    read_arg("package", &mut result, |arg| default_package::get_package(&result["group"], &result["artifact"]), iterative)?;
    result.insert("packagePath".to_string(), result["package"].replace('.', "/"));

    if let Some(target_jvm_info) = target_jvm {
        read_arg("targetJvm", &mut result, optional_default(target_jvm_info.default), iterative)?;
        result.insert("targetJvmJava".to_string(), result["targetJvm"].replace('.', "_"));
    }

    for (arg_name, arg_info) in args {
        read_arg(&arg_name, &mut result, optional_default(arg_info.default), iterative)?;
    }
    result.insert("dollar".to_string(), "$".to_string());
    Ok(result)
}

pub(super) fn print_args_list(args_info: Args) {
    println!("    group: Project groupId
    artifact: Project artifactId
    package: Project root package. If not specified, use <group>.<artifact> and try to fix the illegal part
    version: Project version");
    if let Some(target_jvm) = args_info.target_jvm {
        print!("    targetJvm: Target JVM version");
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
