use super::TemplateFile;

pub(super) const GITATTRIBUTES: TemplateFile = TemplateFile {
    path: |_| ".gitattributes".into(),
    write_content: |_, w| {
        writeln!(w, r"*.bat text eol=crlf")
    },
};