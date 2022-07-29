mod args;
mod template_file;

use crate::templates::template_file::{Template, TemplateInfoOnly};
use crate::{log_error, log_warn, Logged};
use fxhash::FxHashMap;
use std::ffi::OsString;
use std::path::PathBuf;

use self::template_file::TemplateDescriptionOnly;

type IndexMapString<V> = indexmap::IndexMap<String, V, fxhash::FxBuildHasher>;

fn get_template_paths() -> Vec<PathBuf> {
    let mut paths = vec![];
    if let Ok(path_current) = std::env::current_exe() {
        let mut path = path_current;
        path.pop();
        path.push("templates");
        paths.push(path);
    }
    if let Some(home) = dirs::home_dir() {
        let mut path = home;
        path.push(".gur/templates");
        paths.push(path);
    }
    if let Ok(paths_str) = std::env::var("GUR_TEMPLATES_PATH") {
        for path in paths_str.split(':') {
            paths.push(PathBuf::from(path));
        }
    }
    paths
}

fn load_templates_info(paths: &[PathBuf]) -> FxHashMap<String, TemplateDescriptionOnly> {
    let mut templates = FxHashMap::default();

    for path in paths {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        if ext == "toml" {
                            let template_name =
                                path.file_stem().unwrap().to_string_lossy().to_string();
                            if !templates.contains_key(&template_name) {
                                if let Ok(file_content) = std::fs::read(path) {
                                    if let Ok(template) = toml::from_slice(&file_content) {
                                        templates.insert(template_name, template);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    templates
}

fn load_template<D>(paths: Vec<PathBuf>, template: &str) -> Result<(D, PathBuf), Logged>
where
    for<'a> D: serde::Deserialize<'a>,
{
    for path in paths {
        let mut path = path;
        path.push(template);
        path.set_extension("toml");
        match std::fs::read(&path) {
            Ok(file_content) => {
                return toml::from_slice(&file_content)
                    .map(|template| (template, path))
                    .map_err(|err| log_error(format_args!("template invalid: {}", err)))
            }
            Err(err) => {
                if err.kind() != std::io::ErrorKind::NotFound {
                    return Err(log_error(format_args!(
                        "cannot open template file, {}",
                        err
                    )));
                }
            }
        }
    }
    Err(log_error(format_args!("tempate `{template}` not found")))
}

fn load_shared(template_path: PathBuf) -> FxHashMap<OsString, Vec<u8>> {
    let mut result = FxHashMap::default();

    let mut shared_path = template_path;
    shared_path.pop();
    shared_path.push("shared");
    if let Ok(entries) = std::fs::read_dir(shared_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(template_name) = path.file_name() {
                    if !result.contains_key(template_name) {
                        if let Ok(file_content) = std::fs::read(&path) {
                            result.insert(template_name.to_os_string(), file_content);
                        }
                    }
                }
            }
        }
    }
    result
}
pub fn list(name: Option<String>) -> Result<(), Logged> {
    if let Some(name) = name {
        let (template, _) = load_template::<TemplateInfoOnly>(get_template_paths(), &name)?;
        println!("Template: {}", name);
        println!("Description: {}", template.description);
        println!("Args:");
        args::print_args_list(template.args);
        println!();
    } else {
        for (name, template) in load_templates_info(&get_template_paths()) {
            println!("{}: {}", name, template.description);
        }
    }

    Ok(())
}

fn apply_args(s: &str, args: &FxHashMap<String, String>) -> Result<String, Logged> {
    const TEMPLATE_ISSUE: &str = "This is the issue of the template author.";

    let mut result = String::new();
    let mut rest = s;
    while let Some(i) = rest.find("$(") {
        result.push_str(&rest[..i]);
        rest = &rest[i + 2..];
        if let Some(j) = rest.find(')') {
            let key = &rest[..j];
            if let Some(value) = args.get(key) {
                result.push_str(value);
            } else {
                return Err(log_error(format_args!(
                    "unknown argument in template: {}. {}",
                    key, TEMPLATE_ISSUE
                )));
            }
            rest = &rest[j + 1..];
        } else {
            return Err(log_error(format_args!(
                "illegal argument in template. {}",
                TEMPLATE_ISSUE
            )));
        }
    }
    result.push_str(rest);
    Ok(result)
}
pub fn new(
    name: String,
    output: PathBuf,
    defines: Vec<(String, String)>,
    allow_exists: bool,
    overwrite: bool,
    iterative: bool,
) -> Result<(), Logged> {
    let (template, path) = load_template::<Template>(get_template_paths(), &name)?;

    if allow_exists {
        if output.is_file() {
            return Err(log_error(format_args!("output path is a file")));
        }
    } else if output.exists() {
        return Err(log_error(format_args!("output directory already exists")));
    }
    
    let shared_files = load_shared(path);
    let args = args::get_args(iterative, defines, template.args)?;

    std::fs::create_dir_all(&output)
        .map_err(|e| log_error(format_args!("failed to create output directory: {}", e)))?;
    for (shared_file_name, file_content) in shared_files {
        let mut file_path = PathBuf::from(&output);
        file_path.push(shared_file_name);
        std::fs::write(file_path, file_content)
            .map_err(|e| log_error(format_args!("failed to write file: {}", e)))?;
    }
    for template_file in template.files {
        let mut file_path = PathBuf::from(&output);
        file_path.push(apply_args(&template_file.path, &args)?);
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
        std::fs::write(file_path, apply_args(&template_file.content, &args)?)
            .map_err(|e| log_error(format_args!("failed to write file: {}", e)))?;
    }
    println!(
        "Finished. Please execute `wrapper` gradle task either via gradle CLI or in your IDE."
    );
    Ok(())
}
