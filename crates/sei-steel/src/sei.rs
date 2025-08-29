use std::{collections::BTreeMap, error::Error, sync::LazyLock};

use alloy_evm::{
    revm::{inspector::NoOpInspector, primitives::hardfork::SpecId},
    Database, EthEvmFactory as AlloyEthEvmFactory, EvmFactory as AlloyEvmFactory,
};
use alloy_primitives::{Address, Bytes};
use risc0_steel::{
    config::{ChainSpec, ForkCondition},
    ethereum::{EthBlockHeader, EthChainSpec},
    EvmEnv, EvmFactory, EvmInput,
};
use serde::{Deserialize, Serialize};

use crate::cosmos::CosmosCommitment;

pub static SEI_MAINNET_CHAIN_SPEC: LazyLock<SeiChainSpec> = LazyLock::new(|| ChainSpec {
    chain_id: 1329,
    forks: BTreeMap::from([(SpecId::CANCUN, ForkCondition::Block(0))]),
});

pub static SEI_TESTNET_CHAIN_SPEC: LazyLock<SeiChainSpec> = LazyLock::new(|| ChainSpec {
    chain_id: 1328,
    forks: BTreeMap::from([(SpecId::CANCUN, ForkCondition::Block(0))]),
});

/// [ChainSpec] for Sei.
pub type SeiChainSpec = EthChainSpec;

/// [EvmBlockHeader] for Sei.
pub type SeiBlockHeader = EthBlockHeader;

/// [EvmFactory] for Sei.
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SeiEvmFactory;

impl EvmFactory for SeiEvmFactory {
    type Evm<DB: Database> = <AlloyEthEvmFactory as AlloyEvmFactory>::Evm<DB, NoOpInspector>;
    type Tx = <AlloyEthEvmFactory as AlloyEvmFactory>::Tx;
    type Error<DBError: Error + Send + Sync + 'static> =
        <AlloyEthEvmFactory as AlloyEvmFactory>::Error<DBError>;
    type HaltReason = <AlloyEthEvmFactory as AlloyEvmFactory>::HaltReason;
    type Spec = <AlloyEthEvmFactory as AlloyEvmFactory>::Spec;
    type Header = EthBlockHeader;

    fn new_tx(_address: Address, _data: Bytes) -> Self::Tx {
        unimplemented!("Sei EVM execution is not yet supported")
    }

    fn create_evm<DB: Database>(
        _db: DB,
        _chain_id: u64,
        _spec: Self::Spec,
        _header: &Self::Header,
    ) -> Self::Evm<DB> {
        unimplemented!("Sei EVM execution is not yet supported")
    }
}
