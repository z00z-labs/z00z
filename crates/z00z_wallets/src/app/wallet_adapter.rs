//! Wallet projection adapter backed by the real application service owner.

use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use std::{
    sync::mpsc::{sync_channel, Receiver, SyncSender},
    thread::JoinHandle,
};

use z00z_app_api::{AppError, BoundedId, BoundedText, PageRequest, WalletPage, WalletProjection};

use crate::{rpc::types::wallet::PersistWalletInfo, services::AppService};

use super::redaction::internal_error;

/// Safe projection adapter; wallet secrets never enter the App API.
pub struct WalletAdapter {
    #[cfg(target_arch = "wasm32")]
    owner: Arc<AppService>,
    #[cfg(not(target_arch = "wasm32"))]
    sender: SyncSender<WalletCommand>,
    #[cfg(not(target_arch = "wasm32"))]
    worker: Option<JoinHandle<()>>,
}

impl WalletAdapter {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(owner: Arc<AppService>) -> Result<Self, AppError> {
        let (sender, receiver) = sync_channel(1);
        let worker = std::thread::Builder::new()
            .name("z00z-wallet-owner".to_owned())
            .spawn(move || run_owner(owner, receiver))
            .map_err(|_| internal_error("wallet-owner-thread"))?;
        Ok(Self {
            sender,
            worker: Some(worker),
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(owner: Arc<AppService>) -> Result<Self, AppError> {
        Ok(Self { owner })
    }

    pub fn list(&self, request: &PageRequest) -> Result<WalletPage, AppError> {
        let projections = self.load_projections()?;
        let start = match request.cursor() {
            None => 0,
            Some(cursor) => projections
                .iter()
                .position(|wallet| wallet.id.as_str() == cursor.as_str())
                .map(|index| index + 1)
                .ok_or(AppError::NotFound)?,
        };
        let limit = usize::from(request.limit());
        let end = start.saturating_add(limit).min(projections.len());
        let items = projections[start..end].to_vec();
        let next_cursor = if end < projections.len() {
            items.last().map(|wallet| wallet.id.clone())
        } else {
            None
        };
        WalletPage::new(items, next_cursor)
    }

    pub fn get(&self, id: &BoundedId) -> Result<WalletProjection, AppError> {
        self.load_projections()?
            .into_iter()
            .find(|wallet| wallet.id.as_str() == id.as_str())
            .ok_or(AppError::NotFound)
    }

    pub fn probe(&self) -> Result<(), AppError> {
        self.load_owner().map(|_| ())
    }

    fn load_projections(&self) -> Result<Vec<WalletProjection>, AppError> {
        self.load_owner()?.into_iter().map(project_wallet).collect()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn load_owner(&self) -> Result<Vec<PersistWalletInfo>, AppError> {
        let (response_tx, response_rx) = sync_channel(1);
        self.sender
            .send(WalletCommand::List(response_tx))
            .map_err(|_| internal_error("wallet-owner-send"))?;
        response_rx
            .recv()
            .map_err(|_| internal_error("wallet-owner-receive"))?
            .map_err(|_| internal_error("wallet-owner-read"))
    }

    #[cfg(target_arch = "wasm32")]
    fn load_owner(&self) -> Result<Vec<PersistWalletInfo>, AppError> {
        let _ = &self.owner;
        Err(AppError::CapabilityUnavailable {
            code: BoundedId::new("wallet-read-native")?,
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Drop for WalletAdapter {
    fn drop(&mut self) {
        let _ = self.sender.send(WalletCommand::Stop);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
enum WalletCommand {
    List(SyncSender<Result<Vec<PersistWalletInfo>, ()>>),
    Stop,
}

#[cfg(not(target_arch = "wasm32"))]
fn run_owner(owner: Arc<AppService>, receiver: Receiver<WalletCommand>) {
    let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    else {
        return;
    };
    while let Ok(command) = receiver.recv() {
        match command {
            WalletCommand::List(response) => {
                let result = runtime.block_on(owner.list_wallets()).map_err(|_| ());
                let _ = response.send(result);
            }
            WalletCommand::Stop => break,
        }
    }
}

fn project_wallet(info: PersistWalletInfo) -> Result<WalletProjection, AppError> {
    if info.created_at == 0 {
        return Err(AppError::StaleProjection);
    }
    Ok(WalletProjection {
        id: BoundedId::new(info.id.0)?,
        label: BoundedText::new(info.name)?,
        locked: info.is_locked,
        revision: info.created_at,
    })
}
