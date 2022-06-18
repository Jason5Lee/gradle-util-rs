mod default_package;

use super::template_file::Args;
use crate::{log_error, Logged};
use fxhash::FxHashMap;

pub(super) fn get_args(
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

    let group = result
        .get("group")
        .ok_or_else(|| log_error(format_args!("missing argument: group")))?;
    let artifact = result
        .get("artifact")
        .ok_or_else(|| log_error(format_args!("missing argument: artifact")))?;
    result
        .get("version")
        .ok_or_else(|| log_error(format_args!("missing argument: version")))?;
    let pkg_info = if let Some(package) = result.get("package") {
        default_package::analysis_package(package)?
    } else {
        default_package::get_package(&group, &artifact)?
    };
    result.insert("package".to_string(), pkg_info.package);
    result.insert("packagePath".to_string(), pkg_info.package_path);

    if let Some(target_jvm_info) = target_jvm {
        if let Some(target_jvm) = result.get("targetJvm") {
            let target_jvm_java = target_jvm.replace('.', "_");
            result.insert("targetJvmJava".to_string(), target_jvm_java);
        } else {
            if let Some(default) = target_jvm_info.default {
                let java = default.replace('.', "_");
                result.insert("targetJvm".to_string(), default);
                result.insert("targetJvmJava".to_string(), java);
            } else {
                return Err(log_error(format_args!("missing argument: targetJvm")));
            }
        }
    }

    for (arg_name, arg_info) in args {
        if !result.contains_key(&arg_name) {
            if let Some(default) = arg_info.default {
                result.insert(arg_name, default);
            } else {
                return Err(log_error(format_args!("missing argument: {}", arg_name)));
            }
        }
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
