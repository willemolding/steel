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

use alloy::{
    network::Ethereum,
    providers::{Provider, ProviderBuilder, RootProvider},
};
use alloy_primitives::{BlockHash, BlockNumber};
use anyhow::Result;
use risc0_steel::{
    host::{
        db::{ProofDb, ProviderDb},
        BlockNumberOrTag, EvmEnvBuilder, HostCommit,
    },
    EvmEnv, EvmInput,
};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};
use tendermint_rpc::{Error, HttpClient as TendermintClient, HttpClientUrl};
use url::Url;

use crate::{
    tendermint::{TendermintCommitment, TendermintInput},
    SeiChainSpec, SeiEvmFactory, SeiEvmInput,
};

type HostSeiEvmEnv<P2, C> = SeiEvmEnv<ProofDb<ProviderDb<Ethereum, P2>>, C>;
/// Wrapped [EvmEnv] for Sei chain.
pub struct SeiEvmEnv<D, C> {
    /// Underlying generic environment without a specific commitment.
    inner: EvmEnv<D, SeiEvmFactory, HostCommit<()>>,
    /// Additional sei-specific commitment.
    commit: C,
}

impl<D, C> Deref for SeiEvmEnv<D, C> {
    type Target = EvmEnv<D, SeiEvmFactory, HostCommit<()>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D, C> DerefMut for SeiEvmEnv<D, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl SeiEvmEnv<(), ()> {
    /// Initialize an Sei-specific builder.
    pub fn builder() -> SeiEvmEnvBuilder<PreProviderStage, (), (), ()> {
        SeiEvmEnvBuilder {
            inner: EvmEnv::builder(),
            tendermint_client: (),
            stage: PhantomData,
        }
    }
}

/// Builder for building an [OpEvmEnv] on the host.
///
/// The builder can be created using [OpEvmEnv::builder()].
#[derive(Clone, Debug)]
pub struct SeiEvmEnvBuilder<Stage, P2, Spec, TendermintClient> {
    /// Underlying generic builder with no Beacon API config.
    inner: EvmEnvBuilder<P2, SeiEvmFactory, Spec, ()>,

    /// Tendermint RPC client to fetch headers and proofs if committing via the Tendermint block hash
    tendermint_client: TendermintClient,

    /// Stage of the builder, uses typestate pattern.
    stage: PhantomData<Stage>,
}

/// First stage of a [OpEvmEnvBuilder] before a provider is set.
#[derive(Clone, Debug)]
pub struct PreProviderStage;
/// Second stage of a [OpEvmEnvBuilder] after a provider has been set.
#[derive(Clone, Debug)]
pub struct ProviderStage;

// Callable without chain specification.
impl<Stage, P2, T> SeiEvmEnvBuilder<Stage, P2, (), T> {
    /// Sets the [OpChainSpec].
    pub fn chain_spec(
        self,
        chain_spec: &SeiChainSpec,
    ) -> SeiEvmEnvBuilder<Stage, P2, &SeiChainSpec, T> {
        SeiEvmEnvBuilder {
            inner: self.inner.chain_spec(chain_spec),
            tendermint_client: self.tendermint_client,
            stage: self.stage,
        }
    }
}

// Callable only without a provider
impl<T> SeiEvmEnvBuilder<PreProviderStage, (), (), T> {
    /// Sets the Sei HTTP RPC endpoint that will be used by the [SeiEvmEnv].
    pub fn rpc(self, url: Url) -> SeiEvmEnvBuilder<ProviderStage, RootProvider<Ethereum>, (), T> {
        self.provider(ProviderBuilder::default().connect_http(url))
    }

    /// Sets the [Provider] that will be used by the [SeiEvmEnv].
    pub fn provider<P2>(self, provider: P2) -> SeiEvmEnvBuilder<ProviderStage, P2, (), T>
    where
        P2: Provider<Ethereum> + Clone,
    {
        let inner = EvmEnv::builder().provider(provider.clone());
        SeiEvmEnvBuilder {
            inner,
            tendermint_client: self.tendermint_client,
            stage: PhantomData,
        }
    }
}

impl<P2, Spec, T> SeiEvmEnvBuilder<ProviderStage, P2, Spec, T> {
    pub fn block_number(self, number: u64) -> Self {
        Self {
            inner: self.inner.block_number(number),
            tendermint_client: self.tendermint_client,
            stage: self.stage,
        }
    }

    pub fn block_number_or_tag(self, block: BlockNumberOrTag) -> Self {
        Self {
            inner: self.inner.block_number_or_tag(block),
            tendermint_client: self.tendermint_client,
            stage: self.stage,
        }
    }
}

// Callable only without a provider
impl<P2, Spec> SeiEvmEnvBuilder<ProviderStage, P2, Spec, ()> {
    /// Sets the Sei Tendermint RPC endpoint that will be used by the [SeiEvmEnv].
    pub fn tendermint_rpc<U>(
        self,
        url: U,
    ) -> Result<SeiEvmEnvBuilder<ProviderStage, P2, Spec, TendermintClient>>
    where
        U: TryInto<HttpClientUrl, Error = Error>,
    {
        Ok(self.tendermint_provider(TendermintClient::new(url)?))
    }

    /// Sets the Sei [TendermintClient] that will be used by the [SeiEvmEnv].
    pub fn tendermint_provider(
        self,
        provider: TendermintClient,
    ) -> SeiEvmEnvBuilder<ProviderStage, P2, Spec, TendermintClient> {
        SeiEvmEnvBuilder {
            inner: self.inner,
            tendermint_client: provider,
            stage: PhantomData,
        }
    }
}

/// A Block Identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum BlockId {
    /// A block hash
    Hash(BlockHash),
    /// A block number or tag (e.g. latest)
    Number(BlockNumberOrTag),
}

// Set the commitment block for history proofs
impl<P2, Spec> SeiEvmEnvBuilder<ProviderStage, P2, Spec, TendermintClient> {
    /// Sets the block number for the commitment.
    ///
    /// See [EvmEnvBuilder::commitment_block_hash] for detailed documentation.
    pub fn commitment_block_number(
        self,
        number: BlockNumber,
    ) -> SeiEvmEnvBuilder<ProviderStage, P2, Spec, TendermintClient> {
        self.commitment_block_number_or_tag(BlockNumberOrTag::Number(number))
    }

    pub fn commitment_block_hash(
        self,
        hash: BlockHash,
    ) -> SeiEvmEnvBuilder<ProviderStage, P2, Spec, TendermintClient> {
        self.commitment_block(BlockId::Hash(hash))
    }

    /// Sets the block number or block tag ("latest", "earliest", "pending")  for the commitment.
    ///
    /// See [EvmEnvBuilder::commitment_block_hash] for detailed documentation.
    pub fn commitment_block_number_or_tag(
        self,
        block: BlockNumberOrTag,
    ) -> SeiEvmEnvBuilder<ProviderStage, P2, Spec, TendermintClient> {
        self.commitment_block(BlockId::Number(block))
    }

    fn commitment_block(
        self,
        block: BlockId,
    ) -> SeiEvmEnvBuilder<ProviderStage, P2, Spec, TendermintClient> {
        SeiEvmEnvBuilder {
            inner: self.inner,
            tendermint_client: self.tendermint_client,
            stage: PhantomData,
        }
    }
}

impl<P2> SeiEvmEnvBuilder<ProviderStage, P2, &SeiChainSpec, ()> {
    /// Build when no Tendermint client is provided, will build a commitment to the evm block only.
    pub async fn build(self) -> Result<HostSeiEvmEnv<P2, ()>>
    where
        P2: Provider<Ethereum>,
    {
        Ok(SeiEvmEnv {
            inner: self.inner.build().await?,
            commit: (),
        })
    }
}

impl<P2> SeiEvmEnvBuilder<ProviderStage, P2, &SeiChainSpec, TendermintClient> {
    /// Build when a Tendermint client is provided. In this case it commits to both the evm block and the
    /// Tendermint block.
    pub async fn build(self) -> Result<HostSeiEvmEnv<P2, TendermintCommitment>>
    where
        P2: Provider<Ethereum>,
    {
        Ok(SeiEvmEnv {
            inner: self.inner.build().await?,
            commit: TendermintCommitment::default(), // TODO(willem): build using the tendermint client
        })
    }
}

impl<P2> HostSeiEvmEnv<P2, ()>
where
    P2: Provider<Ethereum>,
{
    pub async fn into_input(self) -> Result<SeiEvmInput> {
        // the inner environment has no specific commitment, so it will always return a block input
        let EvmInput::Block(input) = self.inner.into_input().await? else {
            unreachable!()
        };

        Ok(input)
    }
}

impl<P2> HostSeiEvmEnv<P2, TendermintCommitment>
where
    P2: Provider<Ethereum>,
{
    pub async fn into_input(self) -> Result<TendermintInput<SeiEvmFactory>> {
        // the inner environment has no specific commitment, so it will always return a block input
        let EvmInput::Block(input) = self.inner.into_input().await? else {
            unreachable!()
        };
        // compose with the tendermint commitment
        Ok(TendermintInput::new(input, self.commit))
    }
}
