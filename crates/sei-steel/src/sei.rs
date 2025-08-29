// Copyright 2025 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{error::Error, sync::LazyLock};

use alloy_evm::{
    revm::{inspector::NoOpInspector, primitives::hardfork::SpecId},
    Database, EthEvmFactory as AlloyEthEvmFactory, EvmFactory as AlloyEvmFactory,
};
use alloy_primitives::{Address, Bytes};
use risc0_steel::{
    config::ChainSpec,
    ethereum::{EthBlockHeader, EthChainSpec},
    EvmFactory,
};
use serde::{Deserialize, Serialize};

pub static SEI_MAINNET_CHAIN_SPEC: LazyLock<SeiChainSpec> =
    LazyLock::new(|| ChainSpec::new_single(1329, SpecId::CANCUN));

pub static SEI_TESTNET_CHAIN_SPEC: LazyLock<SeiChainSpec> =
    LazyLock::new(|| ChainSpec::new_single(1328, SpecId::CANCUN));

/// [ChainSpec] for Sei.
pub type SeiChainSpec = EthChainSpec;

/// [EvmBlockHeader] for Sei.
pub type SeiBlockHeader = EthBlockHeader;

pub type SeiEvmInput = risc0_steel::BlockInput<SeiEvmFactory>;

/// [EvmFactory] for Sei.
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SeiEvmFactory;

// This is stubbed out for now until we want to support EVM execution and state reading
// Note this is not trivial as Sei uses a different state tree data structure to Ethereum
impl EvmFactory for SeiEvmFactory {
    type Evm<DB: Database> = <AlloyEthEvmFactory as AlloyEvmFactory>::Evm<DB, NoOpInspector>;
    type Tx = <AlloyEthEvmFactory as AlloyEvmFactory>::Tx;
    type Error<DBError: Error + Send + Sync + 'static> =
        <AlloyEthEvmFactory as AlloyEvmFactory>::Error<DBError>;
    type HaltReason = <AlloyEthEvmFactory as AlloyEvmFactory>::HaltReason;
    type Spec = <AlloyEthEvmFactory as AlloyEvmFactory>::Spec;
    type Header = SeiBlockHeader;

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
