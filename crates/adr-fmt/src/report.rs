//! Diagnostic reporting for ADR validation.

use std::fmt;
use std::path::Path;

/// Diagnostic severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error => write!(f, "error"),
            Self::Warning => write!(f, "warning"),
        }
    }
}

/// A single diagnostic message attached to a file and optional line.
#[derive(Debug)]
pub struct Diagnostic {
    pub severity: Severity,
    pub rule: &'static str,
    pub file: String,
    pub line: usize,
    pub message: String,
    /// Internal diagnostics are self-checks (e.g., I001-I003 verifying
    /// generated README consistency). They are not shown to users.
    pub internal: bool,
}

impl Diagnostic {
    #[allow(dead_code)] // Symmetric with warning(); used by I003 internal_error
    pub fn error(rule: &'static str, file: &Path, line: usize, message: String) -> Self {
        Self {
            severity: Severity::Error,
            rule,
            file: file.display().to_string(),
            line,
            message,
            internal: false,
        }
    }

    pub fn warning(rule: &'static str, file: &Path, line: usize, message: String) -> Self {
        Self {
            severity: Severity::Warning,
            rule,
            file: file.display().to_string(),
            line,
            message,
            internal: false,
        }
    }

    pub fn internal_warning(
        rule: &'static str,
        file: &Path,
        line: usize,
        message: String,
    ) -> Self {
        Self {
            severity: Severity::Warning,
            rule,
            file: file.display().to_string(),
            line,
            message,
            internal: true,
        }
    }

    pub fn internal_error(
        rule: &'static str,
        file: &Path,
        line: usize,
        message: String,
    ) -> Self {
        Self {
            severity: Severity::Error,
            rule,
            file: file.display().to_string(),
            line,
            message,
            internal: true,
        }
    }
}

/// Print a diagnostic to stderr in a compiler-style format.
pub fn print_diagnostic(d: &Diagnostic) {
    if d.line > 0 {
        eprintln!(
            "{severity}[{rule}]: {file}:{line}: {message}",
            severity = d.severity,
            rule = d.rule,
            file = d.file,
            line = d.line,
            message = d.message,
        );
    } else {
        eprintln!(
            "{severity}[{rule}]: {file}: {message}",
            severity = d.severity,
            rule = d.rule,
            file = d.file,
            message = d.message,
        );
    }
}
