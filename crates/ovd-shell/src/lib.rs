//! The shell state and the hull views of the web build (WebAssembly), kept free of any platform so a
//! paused desktop app can return over it (ADR 0026). No calculation: every answer comes from `ovd-format`.

pub mod damage;
pub mod state;
mod vessel;
pub mod views;

pub use state::AppState;
