use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

/// A domain directory (e.g., `docs/adr/cherry/` with prefix `CHE`).
#[derive(Debug, Clone)]
pub struct DomainDir {
    pub path: PathBuf,
    pub prefix: String,
    pub name: String,
    #[allow(dead_code)] // Retained for guidelines output
    pub description: String,
}

/// A tagged rule extracted from the Decision section.
///
/// Format in ADR: `- **R1**: Rule text here`
/// Global identifier: `CHE-0042:R1`
#[derive(Debug, Clone)]
pub struct TaggedRule {
    pub id: String,
    pub text: String,
    #[allow(dead_code)] // Line number for diagnostic context
    pub line: usize,
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
    pub has_retirement: bool,
    #[allow(dead_code)] // Parsed but unused — all terminal states use Retirement
    pub has_rejection_rationale: bool,
    /// True when the ADR file lives in the stale archive directory.
    pub is_stale: bool,
    /// True when the ADR has a `- Root: SELF` self-reference.
    #[allow(dead_code)] // Used by generate tree logic (retained for future)
    pub is_self_referencing: bool,
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
    #[allow(dead_code)] // Parsed for completeness
    pub related_has_placeholder: bool,
    /// Ordered list of H2 section names as they appear in the file.
    pub section_order: Vec<String>,
    /// Word count per H2 section (section name → count). Code blocks
    /// are excluded from the count.
    pub section_word_counts: HashMap<String, usize>,
    /// Crates associated with this ADR via `Crates:` metadata field.
    pub crates: Vec<String>,
    /// Tagged rules extracted from the Decision section
    /// (`- **RN**: text` pattern). Falls back to R0 with full
    /// decision content when no tagged rules are found.
    pub decision_rules: Vec<TaggedRule>,
    /// Full text of the Decision section (for R0 fallback).
    #[allow(dead_code)] // Available for context mode R0 extraction
    pub decision_content: Option<String>,
}

impl Default for AdrRecord {
    fn default() -> Self {
        Self {
            id: AdrId {
                prefix: String::new(),
                number: 0,
            },
            file_path: PathBuf::new(),
            title: None,
            title_line: 0,
            date: None,
            date_line: 0,
            last_reviewed: None,
            last_reviewed_line: 0,
            tier: None,
            tier_line: 0,
            status: None,
            status_line: 0,
            status_raw: None,
            relationships: Vec::new(),
            has_related: false,
            has_context: false,
            has_decision: false,
            has_consequences: false,
            has_retirement: false,
            has_rejection_rationale: false,
            is_stale: false,
            is_self_referencing: false,
            max_code_block_lines: 0,
            max_code_block_line: 0,
            code_block_count: 0,
            amendment_dates: Vec::new(),
            related_has_placeholder: false,
            section_order: Vec::new(),
            section_word_counts: HashMap::new(),
            crates: Vec::new(),
            decision_rules: Vec::new(),
            decision_content: None,
        }
    }
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

    /// Human-readable tier name.
    pub fn name(self) -> &'static str {
        match self {
            Self::S => "Foundational",
            Self::A => "Core",
            Self::B => "Behavioural",
            Self::C => "Tooling",
            Self::D => "Detail",
        }
    }

    /// Tier meaning and scope description.
    pub fn description(self) -> &'static str {
        match self {
            Self::S => "Design philosophy or architecture pattern — changing \
                        reverberates through every crate and every downstream consumer.",
            Self::A => "Core trait design or invariant — changing requires major \
                        refactoring across multiple crates.",
            Self::B => "Behavioural contracts and API semantics — changing requires \
                        coordinated updates across call sites.",
            Self::C => "Tooling, DX, and build decisions — changing is localized to \
                        configuration or test infrastructure.",
            Self::D => "Implementation detail — changing affects one crate's internals.",
        }
    }

    /// Stability expectation for this tier.
    pub fn stability(self) -> &'static str {
        match self {
            Self::S => "Immutable post-1.0",
            Self::A => "Near-immutable; changes require RFC-level discussion",
            Self::B => "Stable; changes documented via Amended status",
            Self::C => "Flexible; changes append monotonically",
            Self::D => "Mutable; may be superseded freely",
        }
    }

    /// Assignment guide — the question to ask when choosing a tier.
    pub fn assignment_guide(self) -> &'static str {
        match self {
            Self::S => "If this changed, would we need to rewrite the framework?",
            Self::A => "If this changed, would trait signatures or type bounds change?",
            Self::B => "If this changed, would call sites or runtime behaviour change?",
            Self::C => "If this changed, would only CI, lints, or test setup change?",
            Self::D => "If this changed, would only one crate's internal implementation change?",
        }
    }

    /// All tier variants in order.
    pub fn all() -> &'static [Self] {
        &[Self::S, Self::A, Self::B, Self::C, Self::D]
    }

    /// Numeric rank for sorting (S=0, A=1, ... D=4).
    pub fn rank(self) -> u8 {
        match self {
            Self::S => 0,
            Self::A => 1,
            Self::B => 2,
            Self::C => 3,
            Self::D => 4,
        }
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
    Rejected,
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
        if trimmed == "Rejected" {
            return Self::Rejected;
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
    /// (e.g., `Accepted (note)`), which is not a valid status format.
    pub fn has_parenthetical(raw: &str) -> bool {
        let trimmed = raw.trim();
        // Check for `(` after the status keyword
        trimmed.contains('(') && trimmed.contains(')')
    }

    /// Returns true for terminal lifecycle states: Rejected, Deprecated,
    /// Superseded. Terminal-state ADRs must be in the stale directory
    /// and have a `## Retirement` section.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Rejected | Self::Deprecated | Self::SupersededBy(_)
        )
    }

    /// Human-readable description of this lifecycle state.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Draft => "Under development, not yet proposed for review. May be incomplete.",
            Self::Proposed => "Ready for review. All required fields present.",
            Self::Accepted => "Decision is binding. Implementation may be pending.",
            Self::Amended { .. } => "Accepted with recorded modifications. Previous text preserved.",
            Self::Rejected => "Decision was proposed but deliberately not adopted. \
                              Remains in record for context.",
            Self::Deprecated => "No longer applicable but preserved for historical context.",
            Self::SupersededBy(_) => "Replaced by another ADR. The superseding ADR is authoritative.",
            Self::Invalid(_) => "Unrecognized status value.",
        }
    }

    /// All recognized status variant names for documentation.
    #[allow(dead_code)] // Used by --guidelines
    pub fn all_variant_names() -> &'static [&'static str] {
        &[
            "Draft",
            "Proposed",
            "Accepted",
            "Amended [YYYY-MM-DD — note]",
            "Rejected",
            "Deprecated",
            "Superseded by PREFIX-NNNN",
        ]
    }

    /// Short display string for output formatting.
    pub fn short_display(&self) -> String {
        match self {
            Self::Draft => "Draft".into(),
            Self::Proposed => "Proposed".into(),
            Self::Accepted => "Accepted".into(),
            Self::Amended { .. } => "Amended".into(),
            Self::Rejected => "Rejected".into(),
            Self::Deprecated => "Deprecated".into(),
            Self::SupersededBy(id) => format!("Superseded by {id}"),
            Self::Invalid(s) => s.clone(),
        }
    }
}

/// A typed, directional relationship between two ADRs.
#[derive(Debug, Clone)]
pub struct Relationship {
    pub verb: RelVerb,
    pub target: AdrId,
    pub line: usize,
}

/// Relationship verb vocabulary.
///
/// Three permitted verbs:
/// - `References` — soft citation (citing → cited)
/// - `Supersedes` — replaces target entirely (newer → older)
/// - `Root` — self-reference marking this ADR as a tree root
///
/// Legacy verbs are retained so the parser can recognize them and
/// L006 can produce migration warnings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelVerb {
    // === Permitted verbs ===
    References,
    Supersedes,
    Root,

    // === Legacy verbs (L006 warns on these) ===
    DependsOn,
    Extends,
    Illustrates,
    ContrastsWith,
    ScopedBy,

    // === Legacy reverse verbs (also L006) ===
    Informs,
    ExtendedBy,
    IllustratedBy,
    ReferencedBy,
    SupersededBy,
    Scopes,
}

impl RelVerb {
    /// True for the three permitted verbs.
    pub fn is_permitted(self) -> bool {
        matches!(self, Self::References | Self::Supersedes | Self::Root)
    }

    /// True for legacy reverse verbs.
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

    /// Human-readable description of the verb's meaning.
    pub fn description(self) -> &'static str {
        match self {
            Self::Root => "Self-reference marking this ADR as a tree root",
            Self::References => "This ADR cites the target in context or consequences",
            Self::Supersedes => "Replaces target entirely; target becomes Deprecated/Superseded",
            _ => "Legacy verb — migrate to a permitted verb",
        }
    }

    /// Migration guidance for legacy verbs. Returns None for permitted verbs.
    pub fn migration(self) -> Option<&'static str> {
        match self {
            Self::DependsOn => Some("use References"),
            Self::Extends => Some("use References"),
            Self::Illustrates => Some("use References"),
            Self::ContrastsWith => Some("use References"),
            Self::ScopedBy => Some("use References"),
            Self::Informs => Some("remove (reverse verb)"),
            Self::ExtendedBy => Some("remove (reverse verb)"),
            Self::IllustratedBy => Some("remove (reverse verb)"),
            Self::ReferencedBy => Some("remove (reverse verb)"),
            Self::SupersededBy => Some("remove (reverse verb)"),
            Self::Scopes => Some("remove (reverse verb)"),
            _ => None,
        }
    }

    /// All permitted verb variants.
    pub fn permitted() -> &'static [Self] {
        &[Self::Root, Self::References, Self::Supersedes]
    }

    /// All legacy verb variants.
    pub fn legacy() -> &'static [Self] {
        &[
            Self::DependsOn,
            Self::Extends,
            Self::Illustrates,
            Self::ContrastsWith,
            Self::ScopedBy,
            Self::Informs,
            Self::ExtendedBy,
            Self::IllustratedBy,
            Self::ReferencedBy,
            Self::SupersededBy,
            Self::Scopes,
        ]
    }

    /// Parse a verb string from the `## Related` section.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "Root" => Some(Self::Root),
            "References" => Some(Self::References),
            "Supersedes" => Some(Self::Supersedes),
            "Depends on" => Some(Self::DependsOn),
            "Informs" => Some(Self::Informs),
            "Extends" => Some(Self::Extends),
            "Extended by" => Some(Self::ExtendedBy),
            "Illustrates" => Some(Self::Illustrates),
            "Illustrated by" => Some(Self::IllustratedBy),
            "Referenced by" => Some(Self::ReferencedBy),
            "Contrasts with" => Some(Self::ContrastsWith),
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
            Self::Root => "Root",
            Self::References => "References",
            Self::Supersedes => "Supersedes",
            Self::DependsOn => "Depends on",
            Self::Informs => "Informs",
            Self::Extends => "Extends",
            Self::ExtendedBy => "Extended by",
            Self::Illustrates => "Illustrates",
            Self::IllustratedBy => "Illustrated by",
            Self::ReferencedBy => "Referenced by",
            Self::ContrastsWith => "Contrasts with",
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
    fn permitted_verbs() {
        assert!(RelVerb::Root.is_permitted());
        assert!(RelVerb::References.is_permitted());
        assert!(RelVerb::Supersedes.is_permitted());
    }

    #[test]
    fn legacy_verbs_not_permitted() {
        let legacy = [
            RelVerb::DependsOn,
            RelVerb::Extends,
            RelVerb::Illustrates,
            RelVerb::ContrastsWith,
            RelVerb::ScopedBy,
        ];
        for verb in legacy {
            assert!(!verb.is_permitted(), "{verb} should not be permitted");
        }
    }

    #[test]
    fn reverse_verbs_not_permitted() {
        let reverse = [
            RelVerb::Informs,
            RelVerb::ExtendedBy,
            RelVerb::IllustratedBy,
            RelVerb::ReferencedBy,
            RelVerb::SupersededBy,
            RelVerb::Scopes,
        ];
        for verb in reverse {
            assert!(!verb.is_permitted(), "{verb} should not be permitted");
            assert!(verb.is_reverse(), "{verb} should be reverse");
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
        let id = parse_adr_id_from_str("CHE-0021").unwrap();
        assert_eq!(id.number, 21);
    }

    #[test]
    fn parse_status_accepted() {
        assert_eq!(Status::parse("Accepted"), Status::Accepted);
    }

    #[test]
    fn parse_status_rejected() {
        assert_eq!(Status::parse("Rejected"), Status::Rejected);
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
        assert!(matches!(s, Status::Invalid(_)));
    }

    #[test]
    fn has_parenthetical_detects_annotations() {
        assert!(Status::has_parenthetical("Accepted (note)"));
        assert!(!Status::has_parenthetical("Accepted"));
        assert!(!Status::has_parenthetical("Amended 2026-04-25 — note"));
    }

    #[test]
    fn root_verb_parse_and_display() {
        assert_eq!(RelVerb::parse("Root"), Some(RelVerb::Root));
        assert_eq!(RelVerb::Root.to_string(), "Root");
    }

    #[test]
    fn verb_display_roundtrip() {
        let verbs = [
            ("Root", RelVerb::Root),
            ("References", RelVerb::References),
            ("Supersedes", RelVerb::Supersedes),
            ("Depends on", RelVerb::DependsOn),
            ("Informs", RelVerb::Informs),
            ("Extends", RelVerb::Extends),
            ("Extended by", RelVerb::ExtendedBy),
            ("Illustrates", RelVerb::Illustrates),
            ("Illustrated by", RelVerb::IllustratedBy),
            ("Referenced by", RelVerb::ReferencedBy),
            ("Contrasts with", RelVerb::ContrastsWith),
            ("Superseded by", RelVerb::SupersededBy),
            ("Scopes", RelVerb::Scopes),
            ("Scoped by", RelVerb::ScopedBy),
        ];
        for (text, verb) in verbs {
            assert_eq!(RelVerb::parse(text), Some(verb), "parse({text})");
            assert_eq!(verb.to_string(), text, "display({verb:?})");
        }
    }

    #[test]
    fn tier_descriptions_non_empty() {
        for tier in Tier::all() {
            assert!(!tier.name().is_empty(), "{tier:?} name");
            assert!(!tier.description().is_empty(), "{tier:?} description");
            assert!(!tier.stability().is_empty(), "{tier:?} stability");
            assert!(!tier.assignment_guide().is_empty(), "{tier:?} guide");
        }
    }

    #[test]
    fn status_is_terminal() {
        assert!(!Status::Draft.is_terminal());
        assert!(!Status::Proposed.is_terminal());
        assert!(!Status::Accepted.is_terminal());
        assert!(
            !Status::Amended {
                date: None,
                note: None
            }
            .is_terminal()
        );
        assert!(Status::Rejected.is_terminal());
        assert!(Status::Deprecated.is_terminal());
        assert!(
            Status::SupersededBy(AdrId {
                prefix: "CHE".into(),
                number: 1
            })
            .is_terminal()
        );
    }

    #[test]
    fn status_descriptions_non_empty() {
        let variants = [
            Status::Draft,
            Status::Proposed,
            Status::Accepted,
            Status::Amended {
                date: None,
                note: None,
            },
            Status::Rejected,
            Status::Deprecated,
            Status::SupersededBy(AdrId {
                prefix: "CHE".into(),
                number: 1,
            }),
            Status::Invalid("bad".into()),
        ];
        for status in &variants {
            assert!(
                !status.description().is_empty(),
                "{status:?} description is empty"
            );
        }
    }

    #[test]
    fn verb_migration_for_legacy() {
        for verb in RelVerb::legacy() {
            assert!(
                verb.migration().is_some(),
                "{verb:?} should have migration guidance"
            );
        }
    }

    #[test]
    fn verb_migration_none_for_permitted() {
        for verb in RelVerb::permitted() {
            assert!(
                verb.migration().is_none(),
                "{verb:?} should not have migration guidance"
            );
        }
    }

    #[test]
    fn default_adr_record() {
        let record = AdrRecord::default();
        assert_eq!(record.id.prefix, "");
        assert_eq!(record.id.number, 0);
        assert!(record.crates.is_empty());
        assert!(record.decision_rules.is_empty());
        assert!(record.decision_content.is_none());
    }

    #[test]
    fn tier_rank_ordering() {
        assert!(Tier::S.rank() < Tier::A.rank());
        assert!(Tier::A.rank() < Tier::B.rank());
        assert!(Tier::D.rank() == 4);
    }

    #[test]
    fn status_short_display() {
        assert_eq!(Status::Draft.short_display(), "Draft");
        assert_eq!(Status::Accepted.short_display(), "Accepted");
        assert_eq!(
            Status::SupersededBy(AdrId {
                prefix: "CHE".into(),
                number: 99,
            })
            .short_display(),
            "Superseded by CHE-0099"
        );
    }
}
