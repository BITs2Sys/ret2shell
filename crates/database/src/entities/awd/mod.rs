//! `SeaORM` entities for the AWD (Attack-and-Defense) and AWDP
//! (Attack-and-Defense-Plus) challenge modes. Fork-owned submodule, registered
//! append-only via one `pub mod awd;` at the end of `entities/mod.rs`.

pub mod awd_instance;
pub mod awd_round;
pub mod awd_state;
pub mod awd_steal;
pub mod awdp_award;
pub mod awdp_solve;
pub mod awdp_state;
