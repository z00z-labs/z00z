use z00z_core::{assets::AssetClass, AssetWire};
use z00z_storage::settlement::{DefinitionId, SerialId, SettlementPath, StoreItem, TerminalId};
use z00z_utils::rng::DeterministicRngProvider;
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    receiver::ReceiverCard,
    tx::{asset_wire_to_leaf, TxOutRole},
};

use super::{config::Scenario2Cfg, runner::Scenario2Err, tx_batch::output_wire, types::OwnedCoin};

#[derive(Clone)]
pub(super) struct WalletSpec {
    pub name: String,
    pub secret: [u8; 32],
    pub card: ReceiverCard,
}

pub(super) struct WalletRing {
    route: Vec<WalletSpec>,
    fee: WalletSpec,
}

impl WalletRing {
    pub fn new(config: &Scenario2Cfg) -> Result<Self, Scenario2Err> {
        let a = wallet("A", config.scenario.seed, 1)?;
        let b = wallet("B", config.scenario.seed, 2)?;
        let c = wallet("C", config.scenario.seed, 3)?;
        let fee = wallet("FEE", config.scenario.seed, 4)?;
        Ok(Self {
            route: vec![a.clone(), b.clone(), c, b, a],
            fee,
        })
    }

    pub fn edge(&self, height: u64) -> (&WalletSpec, &WalletSpec) {
        let edge_count = self.route.len() - 1;
        let index = usize::try_from(height.saturating_sub(1)).unwrap_or(0) % edge_count;
        (&self.route[index], &self.route[index + 1])
    }

    pub const fn fee(&self) -> &WalletSpec {
        &self.fee
    }

    pub fn seed_coins(&self, config: &Scenario2Cfg) -> Result<Vec<OwnedCoin>, Scenario2Err> {
        let wallet_a = &self.route[0];
        (0..config.load.transactions_per_block)
            .map(|lane| {
                let serial = lane
                    .checked_add(1)
                    .ok_or_else(|| Scenario2Err::Config("seed serial overflow".to_string()))?;
                let seed = tx_seed(config.scenario.seed, 0, lane, b"genesis-output");
                let mut rng = DeterministicRngProvider::from_seed(seed).rng();
                let bundle = z00z_wallets::build_output_bundle_with_rng(
                    wallet_a.name.clone(),
                    TxOutRole::Recipient,
                    AssetClass::Coin,
                    &wallet_a.card,
                    config.load.initial_value_per_lane,
                    serial,
                    &mut rng,
                )
                .map_err(|error| Scenario2Err::Wallet(error.to_string()))?;
                let wire = output_wire(&bundle, 0)?
                    .asset_wire
                    .to_wire()
                    .map_err(|error| Scenario2Err::Wallet(error.to_string()))?;
                owned_coin(lane, wire)
            })
            .collect()
    }
}

pub(super) fn owned_coin(lane: u32, wire: AssetWire) -> Result<OwnedCoin, Scenario2Err> {
    let leaf = asset_wire_to_leaf(&wire).map_err(Scenario2Err::Wallet)?;
    let path = SettlementPath::new(
        DefinitionId::new(wire.definition.id),
        SerialId::new(wire.serial_id),
        TerminalId::new(leaf.asset_id),
    );
    StoreItem::new(path, leaf).map_err(|error| Scenario2Err::Storage(error.to_string()))?;
    Ok(OwnedCoin { lane, wire, path })
}

pub(super) fn coin_item(coin: &OwnedCoin) -> Result<StoreItem, Scenario2Err> {
    let leaf = asset_wire_to_leaf(&coin.wire).map_err(Scenario2Err::Wallet)?;
    StoreItem::new(coin.path, leaf).map_err(|error| Scenario2Err::Storage(error.to_string()))
}

pub(super) fn tx_seed(master: u64, height: u64, lane: u32, label: &[u8]) -> [u8; 32] {
    z00z_crypto::blake2b_hash(
        b"z00z.simulator.scenario-2.rng.v1",
        &[
            &master.to_le_bytes(),
            &height.to_le_bytes(),
            &lane.to_le_bytes(),
            label,
        ],
    )
}

fn wallet(name: &str, seed: u64, ordinal: u8) -> Result<WalletSpec, Scenario2Err> {
    let mut secret = z00z_crypto::blake2b_hash(
        b"z00z.simulator.scenario-2.wallet.v1",
        &[&seed.to_le_bytes(), &[ordinal]],
    );
    if secret == [0; 32] {
        secret[0] = ordinal.max(1);
    }
    let keys = ReceiverKeys::from_receiver_secret(
        ReceiverSecret::from_bytes(secret)
            .map_err(|error| Scenario2Err::Wallet(error.to_string()))?,
    )
    .map_err(|error| Scenario2Err::Wallet(error.to_string()))?;
    let mut rng =
        DeterministicRngProvider::from_seed(tx_seed(seed, 0, u32::from(ordinal), b"card")).rng();
    let card = keys
        .export_receiver_card_with_rng(&mut rng)
        .map_err(|error| Scenario2Err::Wallet(error.to_string()))?;
    Ok(WalletSpec {
        name: name.to_string(),
        secret,
        card,
    })
}
