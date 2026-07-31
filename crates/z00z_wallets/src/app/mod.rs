//! Core application logic and app-owned control boundaries.

mod action_adapter;
mod app_facade;
pub mod app_kernel;
mod authorization;
mod extension_adapter;
mod journal;
mod redaction;
mod wallet_adapter;

pub use app_facade::AppFacade;
pub use app_kernel::{AppKernel, CreateWalletRequest, Z00ZApp};
pub use authorization::{FinalOwner, OwnerRoute, OWNER_ROUTES};
pub use journal::{DurableJournal, JournalRecord};
