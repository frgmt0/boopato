// Re-exports for commands
mod admin;
mod boops;
mod commit;
mod games;
mod jobs;
mod redistribution;
mod work;
mod yappers;

// Re-export command functions for main.rs usage
pub use admin::*;
pub use boops::*;
pub use commit::*;
pub use games::*;
pub use jobs::*;
pub use redistribution::*;
pub use work::*;
pub use yappers::*;

// Re-export command functions with proper SendSync bounds
/*
pub mod impls {
    pub use super::work::work;
    pub use super::yappers::yappers;
    pub use super::commit::commit;
    pub use super::boops::boops;
}
*/ 