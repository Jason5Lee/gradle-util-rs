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

    let mut template = File::create("template.toml")?.apply(BufWriter::new);

    write!(
        template,
        "[args]
# TODO

"
    )?;

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
            r#"[[files]]
path = "{}"
content = """{}"""

"#,
            pathdiff::diff_paths(file.path(), &path).unwrap().display(),
            content,
        )?;
    }
    Ok(())
}
