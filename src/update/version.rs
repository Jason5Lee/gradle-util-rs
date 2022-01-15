use std::fmt::{Display, Formatter};

// Note: it's may not be a strictly valid semvar.
// TODO: support rc, milestone and any other possible version.
#[derive(Copy, Clone)]
pub struct GradleVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: Option<u32>,
}

impl GradleVersion {
    fn parse_number(text: &str) -> Result<u32, GradleVersionParseError> {
        text.parse()
            .map_err(|_| GradleVersionParseError::InvalidNumber(text.to_string()))
    }
    pub fn parse(text: &str) -> Result<GradleVersion, GradleVersionParseError> {
        let p = text.find('.').ok_or(GradleVersionParseError::NoDot)?;
        if p == text.len() {
            return Err(GradleVersionParseError::TrailingDot);
        }

        let major: u32 = Self::parse_number(&text[..p])?;

        let text = &text[(p + 1)..];
        match text.find('.') {
            Some(p) => {
                if p == text.len() {
                    return Err(GradleVersionParseError::TrailingDot);
                }
                Ok(GradleVersion {
                    major,
                    minor: Self::parse_number(&text[..p])?,
                    patch: Some(Self::parse_number(&text[(p + 1)..])?),
                })
            }
            None => Ok(GradleVersion {
                major,
                minor: Self::parse_number(text)?,
                patch: None,
            }),
        }
    }

    pub fn can_replace(&self, old_ver: &GradleVersion) -> bool {
        self.major == old_ver.major
            && self.minor == old_ver.minor
            && match (self.patch, old_ver.patch) {
                (Some(_), None) => true,
                (Some(self_patch), Some(old_patch)) => self_patch > old_patch,
                _ => false,
            }
    }
}

impl Display for GradleVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)?;
        if let Some(patch) = self.patch {
            write!(f, ".{}", patch)?
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum GradleVersionParseError {
    NoDot,
    TrailingDot,
    InvalidNumber(String),
}

impl Display for GradleVersionParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GradleVersionParseError::NoDot => write!(f, "no dot"),
            GradleVersionParseError::TrailingDot => write!(f, "last character shouldn't be a dot"),
            GradleVersionParseError::InvalidNumber(num) => write!(f, "invalid number: {}", num),
        }
    }
}
