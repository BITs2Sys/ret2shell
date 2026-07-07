//! `SeaORM` entities for the ISW (Internal Security Warfare) range mode.
//!
//! Fork-owned submodule. Registered append-only via a single `pub mod isw;` line at
//! the end of `entities/mod.rs` plus a trailing re-export in `lib.rs`, so upstream
//! entity regeneration never collides with the fork.

pub mod isw_assignment;
pub mod isw_flag;
pub mod isw_host;
pub mod isw_range;
pub mod isw_range_template;
pub mod isw_vm;
pub mod isw_vpn_peer;
