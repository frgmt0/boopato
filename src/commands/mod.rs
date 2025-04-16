// Re-exports for commands
mod about;
mod admin;
mod boops;
mod commit;
mod games;
mod jobs;
mod kremlin_secrets;
mod redistribution;
mod soviet_hangman;
mod work;

// Re-export command functions for main.rs usage
pub use about::*;
pub use admin::*;
pub use boops::*;
pub use commit::*;
pub use games::*;
pub use jobs::*;
pub use kremlin_secrets::*;
pub use redistribution::*;
pub use soviet_hangman::*;
pub use work::*;

// Re-export command functions with proper SendSync bounds
/*
pub mod impls {
    pub use super::work::work;
    pub use super::yappers::yappers;
    pub use super::commit::commit;
    pub use super::boops::boops;
}
*/ 