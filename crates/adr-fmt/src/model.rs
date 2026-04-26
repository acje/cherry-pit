use std::fmt;
use std::path::PathBuf;

/// A domain directory (e.g., `docs/adr/cherry/` with prefix `CHE`).
#[derive(Debug, Clone)]
pub struct DomainDir {
    pub path: PathBuf,
    pub prefix: String,
}

/// Composite ADR identifier: prefix + number (e.g., CHE-0042).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AdrId {
    pub prefix: String,
    pub number: u16,
}

impl fmt::Display for AdrId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{:04}", self.prefix, self.number)
    }
}

/// Parsed ADR record with all metadata and line numbers.
#[derive(Debug)]
pub struct AdrRecord {
    pub id: AdrId,
    pub file_path: PathBuf,
    pub title: Option<String>,
    pub title_line: usize,
    pub date: Option<String>,
    #[allow(dead_code)]
    pub date_line: usize,
    pub last_reviewed: Option<String>,
    #[allow(dead_code)]
    pub last_reviewed_line: usize,
    pub tier: Option<Tier>,
    #[allow(dead_code)]
    pub tier_line: usize,
    pub status: Option<Status>,
    pub status_line: usize,
    pub status_raw: Option<String>,
    pub relationships: Vec<Relationship>,
    pub has_related: bool,
    pub has_context: bool,
    pub has_decision: bool,
    pub has_consequences: bool,
    pub max_code_block_lines: usize,
    /// 1-indexed line number of the opening fence of the largest code
    /// block. 0 if no code blocks exist.
    pub max_code_block_line: usize,
    #[allow(dead_code)] // reserved for future T-rules
    pub code_block_count: usize,
    /// All `Amended YYYY-MM-DD — note` dates found in the Status section,
    /// paired with their 1-indexed line numbers.
    pub amendment_dates: Vec<(String, usize)>,
    /// True when the Related section contains a `—` placeholder (no
    /// relationships).
    pub related_has_placeholder: bool,
}

/// ADR tier classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    S,
    A,
    B,
    C,
    D,
}

impl Tier {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "S" => Some(Self::S),
            "A" => Some(Self::A),
            "B" => Some(Self::B),
            "C" => Some(Self::C),
            "D" => Some(Self::D),
            _ => None,
        }
    }

    /// S and A tier ADRs require `Last-reviewed`.
    pub fn requires_last_reviewed(self) -> bool {
        matches!(self, Self::S | Self::A)
    }
}

/// ADR lifecycle status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Draft,
    Proposed,
    Accepted,
    Amended {
        date: Option<String>,
        note: Option<String>,
    },
    Deprecated,
    SupersededBy(AdrId),
    /// Status line could not be parsed into a known variant.
    Invalid(String),
}

impl Status {
    /// Parse a status line. Returns `Invalid` if unrecognized.
    pub fn parse(line: &str) -> Self {
        let trimmed = line.trim();

        if trimmed == "Draft" {
            return Self::Draft;
        }
        if trimmed == "Proposed" {
            return Self::Proposed;
        }
        if trimmed == "Accepted" {
            return Self::Accepted;
        }
        if trimmed == "Deprecated" {
            return Self::Deprecated;
        }

        // "Amended" or "Amended YYYY-MM-DD — note"
        if trimmed == "Amended" {
            return Self::Amended {
                date: None,
                note: None,
            };
        }
        if let Some(rest) = trimmed.strip_prefix("Amended ") {
            let parts: Vec<&str> = rest.splitn(2, " — ").collect();
            let date = Some(parts[0].to_owned());
            let note = parts.get(1).map(|s| (*s).to_owned());
            return Self::Amended { date, note };
        }

        // "Superseded by PREFIX-NNNN"
        if let Some(rest) = trimmed.strip_prefix("Superseded by ") {
            if let Some(id) = parse_adr_id_from_str(rest.trim()) {
                return Self::SupersededBy(id);
            }
        }

        Self::Invalid(trimmed.to_owned())
    }

    /// Returns true if the raw status line has parenthetical content
    /// (e.g., `Accepted (note)`), which violates governance §6.
    pub fn has_parenthetical(raw: &str) -> bool {
        let trimmed = raw.trim();
        // Check for `(` after the status keyword
        trimmed.contains('(') && trimmed.contains(')')
    }
}

/// A typed, directional relationship between two ADRs.
#[derive(Debug, Clone)]
pub struct Relationship {
    pub verb: RelVerb,
    pub target: AdrId,
    pub line: usize,
}

/// The 12-verb relationship vocabulary from GOVERNANCE.md §5.
///
/// 7 forward verbs (child → parent direction) are the only verbs
/// stored in ADR files: `DependsOn`, `Extends`, `Illustrates`,
/// `References`, `ContrastsWith` (symmetric), `Supersedes`, `ScopedBy`.
///
/// 5 reverse variants (`Informs`, `ExtendedBy`, `IllustratedBy`,
/// `ReferencedBy`, `SupersededBy`, `Scopes`) are retained so the
/// parser can recognize them and L005 can produce clear rejection
/// diagnostics. Use `is_reverse()` to distinguish.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelVerb {
    DependsOn,
    Informs,
    Extends,
    ExtendedBy,
    Illustrates,
    IllustratedBy,
    References,
    ReferencedBy,
    ContrastsWith,
    Supersedes,
    SupersededBy,
    Scopes,
    ScopedBy,
}

impl RelVerb {
    /// Return the reverse verb. For every forward link A→B with this
    /// verb, B must have `reverse()` → A.
    #[cfg(test)]
    pub fn reverse(self) -> Self {
        match self {
            Self::DependsOn => Self::Informs,
            Self::Informs => Self::DependsOn,
            Self::Extends => Self::ExtendedBy,
            Self::ExtendedBy => Self::Extends,
            Self::Illustrates => Self::IllustratedBy,
            Self::IllustratedBy => Self::Illustrates,
            Self::References => Self::ReferencedBy,
            Self::ReferencedBy => Self::References,
            Self::ContrastsWith => Self::ContrastsWith,
            Self::Supersedes => Self::SupersededBy,
            Self::SupersededBy => Self::Supersedes,
            Self::Scopes => Self::ScopedBy,
            Self::ScopedBy => Self::Scopes,
        }
    }

    /// Symmetric verbs require the same verb in both directions.
    pub fn is_symmetric(self) -> bool {
        matches!(self, Self::ContrastsWith)
    }

    /// Reverse verbs point away from the dependency root (parent →
    /// child). These are not stored in ADR files — use `adr-fmt
    /// --report` to compute them on demand.
    pub fn is_reverse(self) -> bool {
        matches!(
            self,
            Self::Informs
                | Self::ExtendedBy
                | Self::IllustratedBy
                | Self::ReferencedBy
                | Self::SupersededBy
                | Self::Scopes
        )
    }

    /// Parse a verb string from the `## Related` section.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "Depends on" => Some(Self::DependsOn),
            "Informs" => Some(Self::Informs),
            "Extends" => Some(Self::Extends),
            "Extended by" => Some(Self::ExtendedBy),
            "Illustrates" => Some(Self::Illustrates),
            "Illustrated by" => Some(Self::IllustratedBy),
            "References" => Some(Self::References),
            "Referenced by" => Some(Self::ReferencedBy),
            "Contrasts with" => Some(Self::ContrastsWith),
            "Supersedes" => Some(Self::Supersedes),
            "Superseded by" => Some(Self::SupersededBy),
            "Scopes" => Some(Self::Scopes),
            "Scoped by" => Some(Self::ScopedBy),
            _ => None,
        }
    }
}

impl fmt::Display for RelVerb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::DependsOn => "Depends on",
            Self::Informs => "Informs",
            Self::Extends => "Extends",
            Self::ExtendedBy => "Extended by",
            Self::Illustrates => "Illustrates",
            Self::IllustratedBy => "Illustrated by",
            Self::References => "References",
            Self::ReferencedBy => "Referenced by",
            Self::ContrastsWith => "Contrasts with",
            Self::Supersedes => "Supersedes",
            Self::SupersededBy => "Superseded by",
            Self::Scopes => "Scopes",
            Self::ScopedBy => "Scoped by",
        };
        write!(f, "{s}")
    }
}

/// Parse an ADR ID from a string like `CHE-0042` or `PAR-0006`.
pub fn parse_adr_id_from_str(s: &str) -> Option<AdrId> {
    let s = s.trim();
    let dash = s.find('-')?;
    let prefix = &s[..dash];
    let num_str = &s[dash + 1..];

    // Take only leading digits (ignore trailing annotations)
    let digits: String = num_str.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }

    let number: u16 = digits.parse().ok()?;
    Some(AdrId {
        prefix: prefix.to_owned(),
        number,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reverse_roundtrip() {
        let verbs = [
            RelVerb::DependsOn,
            RelVerb::Informs,
            RelVerb::Extends,
            RelVerb::ExtendedBy,
            RelVerb::Illustrates,
            RelVerb::IllustratedBy,
            RelVerb::References,
            RelVerb::ReferencedBy,
            RelVerb::ContrastsWith,
            RelVerb::Supersedes,
            RelVerb::SupersededBy,
            RelVerb::Scopes,
            RelVerb::ScopedBy,
        ];
        for verb in verbs {
            assert_eq!(
                verb.reverse().reverse(),
                verb,
                "double reverse of {verb} should be identity"
            );
        }
    }

    #[test]
    fn only_contrasts_with_is_symmetric() {
        assert!(RelVerb::ContrastsWith.is_symmetric());
        assert!(!RelVerb::DependsOn.is_symmetric());
        assert!(!RelVerb::References.is_symmetric());
        assert!(!RelVerb::Scopes.is_symmetric());
    }

    #[test]
    fn is_reverse_true_for_reverse_verbs() {
        let reverse = [
            RelVerb::Informs,
            RelVerb::ExtendedBy,
            RelVerb::IllustratedBy,
            RelVerb::ReferencedBy,
            RelVerb::SupersededBy,
            RelVerb::Scopes,
        ];
        for verb in reverse {
            assert!(verb.is_reverse(), "{verb} should be reverse");
        }
    }

    #[test]
    fn is_reverse_false_for_forward_verbs() {
        let forward = [
            RelVerb::DependsOn,
            RelVerb::Extends,
            RelVerb::Illustrates,
            RelVerb::References,
            RelVerb::ContrastsWith,
            RelVerb::Supersedes,
            RelVerb::ScopedBy,
        ];
        for verb in forward {
            assert!(!verb.is_reverse(), "{verb} should not be reverse");
        }
    }

    #[test]
    fn parse_adr_id() {
        let id = parse_adr_id_from_str("CHE-0042").unwrap();
        assert_eq!(id.prefix, "CHE");
        assert_eq!(id.number, 42);
        assert_eq!(id.to_string(), "CHE-0042");
    }

    #[test]
    fn parse_adr_id_with_trailing_text() {
        // IDs with annotations like "CHE-0021 (`#[non_exhaustive]`)"
        let id = parse_adr_id_from_str("CHE-0021").unwrap();
        assert_eq!(id.number, 21);
    }

    #[test]
    fn parse_status_accepted() {
        assert_eq!(Status::parse("Accepted"), Status::Accepted);
    }

    #[test]
    fn parse_status_amended_with_date() {
        let s = Status::parse("Amended 2026-04-25 — added fencing");
        match s {
            Status::Amended { date, note } => {
                assert_eq!(date.as_deref(), Some("2026-04-25"));
                assert_eq!(note.as_deref(), Some("added fencing"));
            }
            other => panic!("expected Amended, got {other:?}"),
        }
    }

    #[test]
    fn parse_status_amended_bare() {
        let s = Status::parse("Amended");
        assert_eq!(
            s,
            Status::Amended {
                date: None,
                note: None
            }
        );
    }

    #[test]
    fn parse_status_superseded() {
        let s = Status::parse("Superseded by CHE-0099");
        match s {
            Status::SupersededBy(id) => {
                assert_eq!(id.prefix, "CHE");
                assert_eq!(id.number, 99);
            }
            other => panic!("expected SupersededBy, got {other:?}"),
        }
    }

    #[test]
    fn parse_status_invalid() {
        let s = Status::parse("Accepted (supersedes original u64 design)");
        // The parenthetical makes it invalid — doesn't match "Accepted" exactly
        assert!(matches!(s, Status::Invalid(_)));
    }

    #[test]
    fn has_parenthetical_detects_annotations() {
        assert!(Status::has_parenthetical("Accepted (note)"));
        assert!(!Status::has_parenthetical("Accepted"));
        assert!(!Status::has_parenthetical("Amended 2026-04-25 — note"));
    }

    #[test]
    fn verb_display_roundtrip() {
        let verbs = [
            ("Depends on", RelVerb::DependsOn),
            ("Informs", RelVerb::Informs),
            ("Extends", RelVerb::Extends),
            ("Extended by", RelVerb::ExtendedBy),
            ("Illustrates", RelVerb::Illustrates),
            ("Illustrated by", RelVerb::IllustratedBy),
            ("References", RelVerb::References),
            ("Referenced by", RelVerb::ReferencedBy),
            ("Contrasts with", RelVerb::ContrastsWith),
            ("Supersedes", RelVerb::Supersedes),
            ("Superseded by", RelVerb::SupersededBy),
            ("Scopes", RelVerb::Scopes),
            ("Scoped by", RelVerb::ScopedBy),
        ];
        for (text, verb) in verbs {
            assert_eq!(RelVerb::parse(text), Some(verb), "parse({text})");
            assert_eq!(verb.to_string(), text, "display({verb:?})");
        }
    }
}
