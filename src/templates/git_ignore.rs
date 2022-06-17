use super::TemplateFile;

pub(super) const GIT_IGNORE: TemplateFile = TemplateFile {
    path: |_| ".gitignore".into(),
    write_content: |_, w| {
        write!(w, r"
")
    },
};