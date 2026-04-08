use std::fmt;

pub enum GenerationKind {
    File { output_file: String },
    Inline { line: usize },
}

pub struct Generation {
    pub source_file: String,
    pub kind: GenerationKind,
}

impl Generation {
    pub fn file(source_file: impl Into<String>, output_file: impl Into<String>) -> Self {
        Self {
            source_file: source_file.into(),
            kind: GenerationKind::File {
                output_file: output_file.into(),
            },
        }
    }

    pub fn inline(source_file: impl Into<String>, line: usize) -> Self {
        Self {
            source_file: source_file.into(),
            kind: GenerationKind::Inline { line },
        }
    }
}

impl fmt::Display for Generation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            GenerationKind::File { output_file } => {
                write!(f, "{} -> {}", self.source_file, output_file)
            }
            GenerationKind::Inline { line } => {
                write!(f, "{}:{}: generated inline block", self.source_file, line)
            }
        }
    }
}
