use super::IndexMapString;

#[derive(serde::Deserialize)]
pub(super) struct ArgInfo {
    pub default: Option<String>,
    pub description: String,
}
#[derive(serde::Deserialize)]
pub(super) struct TargetJvmArgInfo {
    pub default: Option<String>,
}
#[derive(serde::Deserialize)]
pub(super) struct Args {
    #[serde(rename = "targetJvm")]
    pub target_jvm: Option<TargetJvmArgInfo>,
    #[serde(flatten)]
    pub args: IndexMapString<ArgInfo>,
}
#[derive(serde::Deserialize)]
pub(super) struct TemplateInfoOnly {
    pub args: Args,
}
#[derive(serde::Deserialize)]
pub(super) struct Template {
    pub args: Args,
    pub files: Vec<TemplateFile>,
}
#[derive(serde::Deserialize)]
pub(super) struct TemplateFile {
    pub path: String,
    pub content: String,
}
