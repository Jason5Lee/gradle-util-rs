//! Generate template generator codes from project files.

use apply::Apply;
use std::fs::{read_to_string, File};
use std::io;
use std::io::BufWriter;
use std::io::Write;
use walkdir::WalkDir;

fn main() -> io::Result<()> {
    let mut args = std::env::args_os();
    args.next();
    let path = match args.next() {
        Some(path) => path,
        None => {
            eprintln!("Usage: gen_template <path>");
            return Ok(());
        }
    };

    let mut template = File::create("template.rs")?.apply(BufWriter::new);
    write!(template, "use crate::templates::{{ArgInfo, index_map_with_capacity, IndexMapString, Template, TemplateFile}};

pub(super) fn create_template() -> Template {{
    Template {{
        extra_args_info: {{
            let mut info: IndexMapString<ArgInfo> = index_map_with_capacity(5);
            info
        }},
        files: || vec![
")?;

    for file in WalkDir::new(&path) {
        let file = match file {
            Ok(f) => f,
            Err(e) => {
                eprintln!("error: {}", e);
                continue;
            }
        };
        let metadata = match file.metadata() {
            Ok(m) => m,
            Err(e) => {
                eprintln!(
                    "error: could not get the metadata of the file `{}`, {}",
                    file.path().display(),
                    e
                );
                continue;
            }
        };
        if !metadata.is_file() {
            continue;
        }
        let content = match read_to_string(file.path()) {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "error: could not read the file `{}`, {}",
                    file.path().display(),
                    e
                );
                continue;
            }
        };
        let mut literal_suffix = "\"".to_string();
        while content.contains(&literal_suffix) {
            literal_suffix.push('#');
        }
        write!(
            template,
            "            TemplateFile {{
                path: |_| \"{}\".into(),
                write_content: |_, w| {{
                    write!(w, r{}\"{}{})
                }},
            }},
",
            pathdiff::diff_paths(file.path(), &path).unwrap().display(),
            &literal_suffix[1..],
            content.replace('{', "{{").replace('}', "}}"),
            literal_suffix
        )?;
    }
    write!(template, "        ],\n    }}\n}}\n")?;
    Ok(())
}
