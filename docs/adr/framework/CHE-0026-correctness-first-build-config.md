# CHE-0026. Correctness-First Build Configuration

Date: 2026-04-25
Last-reviewed: 2026-04-25
Tier: C

## Status

Accepted

## Related

- Depends on: CHE-0001, CHE-0007
- Illustrates: CHE-0001

## Context

Rust's release profile controls compiler optimizations, debug
information, and runtime checks. Several settings trade correctness
for performance:

- **`overflow-checks`** — when `false` (release default), integer
  overflow wraps silently. When `true`, overflow panics. The
  framework uses `u64` for aggregate IDs, sequence numbers, and
  index arithmetic — silent overflow could cause data corruption.
- **Clippy lint level** — `pedantic` catches subtle correctness
  issues (shadowing, implicit conversions, missing docs) but
  generates many warnings.
- **LTO, strip, codegen-units** — performance and binary size
  optimizations that do not affect correctness.

## Decision

The workspace `Cargo.toml` sets:

```toml
[profile.release]
lto = true
strip = true
codegen-units = 1
overflow-checks = true

[workspace.lints.clippy]
pedantic = { level = "warn", priority = -1 }
```

Key points:

- **`overflow-checks = true`** — integer overflow panics in release
  builds, not just debug builds. Consistent with design priority P1
  (correctness > speed).
- **`clippy::pedantic`** at warn level — applied workspace-wide so
  all crates share the same lint standard.
- **`lto = true` + `codegen-units = 1`** — enables whole-program
  optimization. Longer compile times for release builds; smaller,
  faster binaries.
- **`strip = true`** — removes debug symbols from release binaries.
  Reduces binary size.

## Consequences

- Integer overflow is always caught, in both debug and release. The
  framework's ID generation and sequence arithmetic rely on
  `checked_add` explicitly, but `overflow-checks = true` provides a
  safety net for any unchecked arithmetic that slips through.
- Clippy pedantic generates many warnings. Individual crates can
  suppress specific pedantic lints via `#[allow(clippy::...)]` where
  justified (e.g., `#[allow(clippy::type_complexity)]` on complex
  return types).
- Release build times are longer due to LTO + single codegen unit.
  Acceptable for a framework — release builds are infrequent.
- Tests run in debug mode where overflow already panics. The release
  profile's `overflow-checks = true` is not directly tested by CI
  but provides production-time safety.
