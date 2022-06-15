mod grpc_vertx_kotlin;
mod kotlin_project;
mod project;
mod git_ignore;
mod gitattributes;

use crate::{log_error, log_warn, Logged};
use std::borrow::Cow;
use std::io::{Error, Write};
use std::path::PathBuf;

const JAVA_KEYWORDS: phf::Set<&'static str> = phf::phf_set!{"abstract", "continue", "for", "new", "switch", "assert", "default", "goto", "package", "synchronized", "boolean", "do", "if", "private", "this", "break", "double", "implements", "protected", "throw", "byte", "else", "import", "public", "throws", "case", "enum", "instanceof", "return", "transient", "catch", "extends", "int", "short", "try", "char", "final", "interface", "static", "void", "class", "finally", "long", "strictfp", "volatile", "const", "float", "native", "super", "while"};
const KOTLIN_HARD_KEYWORDS: phf::Set<&'static str> = phf::phf_set!{"as", "break", "class", "continue", "do", "else", "false", "for", "fun", "if", "in", "interface", "is", "null", "object", "package", "return", "super", "this", "throw", "true", "try", "typealias", "typeof", "val", "var", "when", "while"};

const PACKAGE_ERR_PREFIX: &str = "default package name invalid and cannot be fixed";
const PACKAGE_ERR_SUFFIX: &str = "please assign the package";

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
    artifact: String,
    package: String,
    package_path: String,
    version: String,
    jvm: String,
    java_jvm: String,
    extras: IndexMapString<String>,
}
#[derive(Default)]
struct PackageInfo {
    package: String,
    package_path: String,
}
impl Args {
    fn from_vec(args: Vec<(String, String)>) -> Result<Self, Logged> {
        let mut r = Self {
            gradle: "".to_string(),
            group: "".to_string(),
            artifact: "".to_string(),
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
                "artifact" => r.artifact = v,
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
        if r.artifact.is_empty() {
            return Err(log_error(format_args!("missing argument: artifact")));
        }
        if r.version.is_empty() {
            return Err(log_error(format_args!("missing argument: version")));
        }
        let PackageInfo { package, package_path } = if r.package.is_empty() {
            Self::get_package(&r.group, &r.artifact)
        } else {
            Self::analysis_package(&r.package)
        }?;
        r.package = package;
        r.package_path = package_path;

        if r.jvm.is_empty() {
            r.jvm = "17".to_string();
        }
        r.java_jvm = r.jvm.replace('.', "_");
        Ok(r)
    }
    fn analysis_package(package: &str) -> Result<PackageInfo, Logged> {
        let mut package_info = PackageInfo::default();
        let mut first = true;

        for item in package.split('.') {
            let item = Self::fix_package_item(item)?;
            if !first {
                package_info.package.push('.');
                package_info.package_path.push('/');
            }
            first = false;

            package_info.package.push_str(&item);
            package_info.package_path.push_str(&item);
        }
        Ok(package_info)
    }
    fn get_package(group: &str, artifact: &str) -> Result<PackageInfo, Logged> {
        let mut package_info = PackageInfo::default();

        for group_path in group.split('.') {
            let item = Self::fix_package_item(group_path)?;

            package_info.package.push_str(&item);
            package_info.package.push('.');

            package_info.package_path.push_str(&item);
            package_info.package_path.push('/');
        }
        let item = Self::fix_package_item(artifact)?;
        package_info.package.push_str(&item);
        package_info.package_path.push_str(&item);
        Ok(package_info)
    }

    // Fix the package item https://docs.oracle.com/javase/tutorial/java/package/namingpkgs.html
    // It assumes the `item` is either extracted from a valid group or a valid name.
    // Which means it should not be empty.
    fn fix_package_item(item: &str) -> Result<String, Logged> {
        // Note: we can actually use kotlin keyword in package,
        // but in the code it need to be wrapped in ``.
        // This makes the logic very complicated so we just change it.
        if JAVA_KEYWORDS.contains(item) || KOTLIN_HARD_KEYWORDS.contains(item) {
            return Ok(format!("{}_", item))
        }

        let mut item_chars = item.chars();
        let first = item_chars.next();
        let mut result: String;
        if let Some(first) = first {
            if first.is_numeric() {
                result = String::with_capacity(item.len() + 1);
                result.push('_');
                result.push(first);
            } else {
                result = String::with_capacity(item.len());
                result.push(Self::fix_non_numeric_char(first)?);
            }
        } else {
            return Err(log_error(format_args!("{}, empty item found in package, {}", PACKAGE_ERR_PREFIX, PACKAGE_ERR_SUFFIX)))
        };

        for ch in item_chars {
            if ch.is_numeric() {
                result.push(ch)
            } else {
                result.push(Self::fix_non_numeric_char(ch)?)
            }
        }

        Ok(result)
    }

    fn fix_non_numeric_char(ch: char) -> Result<char, Logged> {
        if ch.is_alphabetic() {
            Ok(ch.to_ascii_lowercase())
        } else if ch == '-' {
            Ok('_')
        } else {
            Err(log_error(format_args!("{}, invalid char `{}` found in package, {}", PACKAGE_ERR_PREFIX, ch, PACKAGE_ERR_SUFFIX)))
        }
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
    group: Project groupId
    artifact: Project artifactId
    package: Project root package, default to <group>.<artifact> and tries to fix the invalid part
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

    let templates = get_templates();
    let template = templates
        .get(&name)
        .ok_or_else(|| log_error(format_args!("template {} not found", name)))?;
    let template: Template = template();

    let mut args = Args::from_vec(defines)?;
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
