mod grpc_vertx_kotlin;
mod kotlin_project;
mod project;
mod git_ignore;
mod gitattributes;

use crate::{log_error, log_warn, Logged};
use std::borrow::Cow;
use std::io::{Error, Write};
use std::path::PathBuf;

type IndexMapString<V> = indexmap::IndexMap<String, V, fxhash::FxBuildHasher>;
fn index_map_with_capacity<V>(size: usize) -> IndexMapString<V> {
    IndexMapString::with_capacity_and_hasher(size, fxhash::FxBuildHasher::default())
}

struct TemplateFile {
    pub path: fn(&Args) -> Cow<'static, str>,
    pub write_content: fn(&Args, w: &mut dyn Write) -> Result<(), Error>,
}

struct Template {
    pub extra_args_info: IndexMapString<ArgInfo>,
    pub files: fn() -> Vec<TemplateFile>,
}

struct Args {
    gradle: String,
    group: String,
    name: String,
    package: String,
    package_path: String,
    version: String,
    jvm: String,
    java_jvm: String,
    extras: IndexMapString<String>,
}
impl Args {
    fn from_vec(args: Vec<(String, String)>) -> Result<Self, Logged> {
        let mut r = Self {
            gradle: "".to_string(),
            group: "".to_string(),
            name: "".to_string(),
            package: "".to_string(),
            package_path: "".to_string(),
            version: "".to_string(),
            jvm: "".to_string(),
            java_jvm: "".to_string(),
            extras: index_map_with_capacity(0),
        };
        for (k, v) in args {
            match k.as_str() {
                "gradle" => r.gradle = v,
                "group" => r.group = v,
                "name" => r.name = v,
                "package" => r.package = v,
                "version" => r.version = v,
                "jvm" => r.jvm = v,
                _ => {
                    r.extras.insert(k, v);
                }
            }
        }
        if r.gradle.is_empty() {
            r.gradle = "7.3.3".to_string();
        }
        if r.group.is_empty() {
            return Err(log_error(format_args!("missing argument: group")));
        }
        if r.name.is_empty() {
            return Err(log_error(format_args!("missing argument: name")));
        }
        if r.version.is_empty() {
            return Err(log_error(format_args!("missing argument: version")));
        }
        if r.package.is_empty() {
            r.package = format!("{}.{}", r.group, r.name);
        }
        r.package_path = r.package.replace('.', "/");
        if r.jvm.is_empty() {
            r.jvm = "17".to_string();
        }
        r.java_jvm = r.jvm.replace('.', "_");
        Ok(r)
    }
}
struct ArgInfo {
    pub default: Option<&'static str>,
    pub description: &'static str,
}

fn get_templates() -> IndexMapString<fn() -> Template> {
    let mut result: IndexMapString<fn() -> Template> = index_map_with_capacity(3);
    result.insert("project".into(), project::create_template);
    result.insert("kotlin-project".into(), kotlin_project::create_template);
    result.insert(
        "grpc-vertx-kotlin".into(),
        grpc_vertx_kotlin::create_template,
    );
    result
}

pub fn list() -> Result<(), Logged> {
    for (name, template) in get_templates() {
        println!("Template: {}", name);
        println!("Args:");
        println!(
            "    gradle: Gradle version, default to 7.3.3
    group: Group name
    name: Project name
    package: Package name, default to <group>.<name>
    version: Project version
    jvm: Java version, default to 17"
        );
        template()
            .extra_args_info
            .iter()
            .for_each(|(k, v)| println!("    {}: {}", k, v.description));
        println!();
    }
    Ok(())
}

pub fn new(
    name: String,
    output: PathBuf,
    defines: Vec<(String, String)>,
    allow_exists: bool,
    overwrite: bool,
) -> Result<(), Logged> {
    if allow_exists {
        if output.is_file() {
            return Err(log_error(format_args!("output path is a file")));
        }
    } else if output.exists() {
        return Err(log_error(format_args!("output directory already exists")));
    }

    let mut args = Args::from_vec(defines)?;
    let templates = get_templates();
    let template = templates
        .get(&name)
        .ok_or_else(|| log_error(format_args!("template {} not found", name)))?;
    let template: Template = template();
    for (k, v) in template.extra_args_info {
        if !args.extras.contains_key(&k) {
            if let Some(d) = v.default {
                args.extras.insert(k, d.to_string());
            } else {
                return Err(log_error(format_args!("missing argument: {}", k)));
            }
        }
    }
    let files = (template.files)();
    for template_file in files {
        let mut file_path = PathBuf::from(&output);
        file_path.push((template_file.path)(&args).as_ref());
        let dir = file_path.parent().unwrap();
        std::fs::create_dir_all(&dir).map_err(|err| {
            log_error(format_args!(
                "failed to create directory {}: {}",
                dir.display(),
                err
            ))
        })?;
        if !overwrite && file_path.exists() {
            log_warn(format_args!(
                "file {} already exists, skipped",
                file_path.display()
            ));
            continue;
        }
        let mut file = std::fs::File::create(&file_path).map_err(|err| {
            log_error(format_args!(
                "failed to create file {}: {}",
                file_path.display(),
                err
            ))
        })?;
        (template_file.write_content)(&args, &mut file).map_err(|err| {
            log_error(format_args!(
                "failed to write file {}: {}",
                file_path.display(),
                err
            ))
        })?;
    }
    println!(
        "Finished. Please execute `wrapper` gradle task either via gradle CLI or in your IDE."
    );
    Ok(())
}
