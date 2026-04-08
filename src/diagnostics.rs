use std::fmt;

#[derive(Clone, Debug)]
pub struct Location {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Clone, Debug)]
pub enum DiagnosticKind {
    MissingInlineGeneration {
        generate_end_byte: usize,
        generated_end_byte: Option<usize>,
        transformed_code: String,
        indentation: usize,
        source_line: usize,
    },
    OutDatedGeneratedCode {
        generate_end_byte: usize,
        generated_end_byte: Option<usize>,
        transformed_code: String,
        indentation: usize,
        source_line: usize,
    },
    GeneratedCodeWithoutGenerator,
    OutOfSyncModuleTarget,
    Warning,
    TagUsedInMarkerNotFoundInConfig {
        tag_name: String,
    },
}

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub message: String,
    pub location: Location,
    pub severity: Severity,
    pub kind: DiagnosticKind,
}

impl Diagnostic {
    pub fn error(
        file: impl Into<String>,
        line: usize,
        column: usize,
        message: impl Into<String>,
        kind: DiagnosticKind,
    ) -> Self {
        Self {
            message: message.into(),
            location: Location {
                file: file.into(),
                line,
                column,
            },
            severity: Severity::Error,
            kind,
        }
    }

    pub fn warning(
        file: impl Into<String>,
        line: usize,
        column: usize,
        message: impl Into<String>,
        kind: DiagnosticKind,
    ) -> Self {
        Self {
            message: message.into(),
            location: Location {
                file: file.into(),
                line,
                column,
            },
            severity: Severity::Warning,
            kind: kind,
        }
    }

    pub fn is_fixable(&self) -> bool {
        matches!(
            self.kind,
            DiagnosticKind::MissingInlineGeneration { .. }
                | DiagnosticKind::OutDatedGeneratedCode { .. }
        )
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let severity = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        };
        write!(
            f,
            "{}:{}:{}: {}: {}",
            self.location.file, self.location.line, self.location.column, severity, self.message
        )
    }
}
