/// Marker trait for commands.
///
/// Commands represent intent — a request to change state. A command
/// may be rejected. Commands are consumed on handling (moved, not
/// borrowed) because they represent a one-time intent.
///
/// Commands entering from remote or retried boundaries should carry an
/// application-level idempotency key in their own payload. The framework
/// deliberately keeps this trait minimal, but duplicate create/ingress
/// commands must have a stable domain key so handlers can return the
/// original effect or no new events after retry.
///
/// # Design rationale
///
/// - Commands are not required to be serializable by default. Only
///   commands that cross process boundaries (via NATS) need serde
///   derives. In-process commands avoid serialization overhead entirely.
/// - The trait is deliberately minimal — a marker with thread-safety
///   bounds. All behavior lives in [`HandleCommand`](crate::HandleCommand).
pub trait Command: Send + Sync + 'static {}
