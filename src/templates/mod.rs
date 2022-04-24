mod project;
mod kotlin_project;
mod grpc_vertx_kotlin;

use std::io::{Write, Error};
use std::path::PathBuf;
use crate::{log_error, log_warn, Logged};

type IndexMapString<V> = indexmap::IndexMap<String, V, fxhash::FxBuildHasher>;
fn index_map_with_capacity<V>(size: usize) -> IndexMapString<V> {
    IndexMapString::with_capacity_and_hasher(size, fxhash::FxBuildHasher::default())
}

struct TemplateFile {
    pub path: &'static str,
    pub write_content: fn(&fxhash::FxHashMap<String, String>, w: &mut dyn Write) -> Result<(), Error>,
}

struct Template {
    pub args: IndexMapString<ArgInfo>,
    pub files: fn() -> Vec<TemplateFile>,
}

struct ArgInfo {
    pub default: Option<&'static str>,
    pub description: &'static str,
}
impl ArgInfo {
    fn with_description(description: &'static str) -> Self {
        ArgInfo {
            default: None,
            description,
        }
    }
}

fn get_templates() -> IndexMapString<fn() -> Template> {
    let mut result: IndexMapString<fn() -> Template> = index_map_with_capacity(3);
    result.insert("project".into(), project::create_template);
    result.insert("kotlin-project".into(), kotlin_project::create_template);
    result.insert("grpc-vertx-kotlin".into(), grpc_vertx_kotlin::create_template);
    result
}

pub fn list() -> Result<(), Logged> {
    for (name, template) in get_templates() {
        println!("Template: {}", name);
        println!("Args:");
        template().args.iter().for_each(|(k, v)| println!("    {}: {}", k, v.description));
        println!();
    }
    Ok(())
}

pub fn new(name: String, output: PathBuf, defines_vec: Vec<(String, String)>, allow_exists: bool, overwrite: bool) -> Result<(), Logged> {
    if allow_exists {
        if output.is_file() {
            return Err(log_error(format_args!("output path is a file")));
        }
    } else {
        return Err(log_error(format_args!("output directory already exists")));
    }
    let mut defines = fxhash::FxHashMap::with_capacity_and_hasher(defines_vec.len(), fxhash::FxBuildHasher::default());
    for (k, v) in defines_vec {
        defines.insert(k, v);
    }
    let templates = get_templates();
    let template = templates.get(&name)
        .ok_or_else(|| log_error(format_args!("template {} not found", name)))?;
    let template: Template = template();
    for (k, v) in template.args {
        if !defines.contains_key(&k) {
            if let Some(default) = v.default {
                defines.insert(k, default.into());
            } else {
                return Err(log_error(format_args!("argument {} is required", k)));
            }
        }
    }
    let files = (template.files)();
    for template_file in files {
        let mut file_path = PathBuf::from(&output);
        file_path.push(template_file.path);
        let dir = file_path.parent().unwrap();
        std::fs::create_dir_all(dir)
            .map_err(|err| log_error(format_args!("failed to create directory {}: {}", dir.display(), err)))?;
        if !overwrite && file_path.exists() {
            log_warn(format_args!("file {} already exists, skipped", file_path.display()));
            continue
        }
        let mut file = std::fs::File::create(&file_path)
            .map_err(|err| log_error(format_args!("failed to create file {}: {}", file_path.display(), err)))?;
        (template_file.write_content)(&defines, &mut file)
            .map_err(|err| log_error(format_args!("failed to write file {}: {}", file_path.display(), err)))?;
    }
    
    Ok(())
}
// TODO: add vert.x kotlin template  
