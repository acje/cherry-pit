//! # cherry-pit-web
//!
//! HTTP adapter family over axum, realising **CHE-0049** (translate
//! HTTP requests to domain commands, dispatch via the gateway, map
//! outcomes to responses with CHE-0039 correlation) and **CHE-0050**
//! (`CommandRouter` port) threaded through `AppState` and
//! [`build_router`] as type parameter `R`.
//!
//! The cqrs router is HTTP-only, no built-in auth (CHE-0049 R2/R3);
//! consumers attach auth via `extra_routes`. [`serve`] exposes the
//! CHE-0086 read-serve surface. Under `feature = "projection"`,
//! [`build_projection_router`] mounts a read side with a narrowed WS
//! upgrade for snapshot-delta push only (CHE-0049 R11).
//!
//! ## Public surface (CHE-0049:R14, CHE-0030:R2)
//!
//! `middleware` is private; its primitives reach consumers via a flat
//! `pub use` here. Remaining surface: [`errors`], [`correlation`],
//! [`path`], [`AppState<G, S, R>`] (CHE-0049:R1, CHE-0050:R2),
//! [`build_router`] (CHE-0049:R9),
//! [`CommandRouter`]/[`DispatchOutcome`] (CHE-0050:R1); under
//! `projection`: [`ProjectionSource`], [`ProjectionState`],
//! [`PageEntry`], [`PageUpdate`], [`build_projection_router`],
//! [`ServerConfig`], [`ServerConfigBuilder`], [`ServerError`],
//! [`ValidatedConfig`], [`ConfigError`].

#![forbid(unsafe_code)]

mod command_router;
pub(crate) mod middleware;
#[cfg(feature = "projection")]
mod projection;
mod router;
pub mod serve;
mod state;

pub mod correlation;
pub mod errors;
pub mod path;

pub use command_router::{CommandRouter, DispatchOutcome};
pub use middleware::{
    HttpTraceLayer, LayerLimits, NormalizedPath, SVG_CSP, WebSocketOriginPolicy, WsPolicy,
    compress_zstd, compute_etag, http_trace_layer, normalize_request_path, sanitize_path_segment,
    security_headers,
};
#[cfg(feature = "projection")]
pub use projection::{
    ConfigError, PageEntry, PageUpdate, ProjectionSource, ProjectionState, ServerConfig,
    ServerConfigBuilder, ServerError, ValidatedConfig, build_projection_router,
};
pub use router::build_router;
pub use serve::{CachedPage, PageUpdateEvent, ServerState, populate_route_aliases};
pub use state::AppState;
