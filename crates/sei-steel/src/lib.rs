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

mod host;
mod sei;
mod tendermint;

pub use host::*;
use risc0_steel::{BlockInput, Commitment, EvmEnv, StateDb};
pub use sei::*; // TODO(willem): Define and restrict public API

pub struct SeiEvmInput {
    block: BlockInput<SeiEvmFactory>,
}

impl SeiEvmInput {
    pub fn new(block: BlockInput<SeiEvmFactory>) -> Self {
        Self { block }
    }
}

impl SeiEvmInput {
    #[inline]
    pub fn into_env(self, chain_spec: &SeiChainSpec) -> EvmEnv<StateDb, SeiEvmFactory, Commitment> {
        self.block.into_env(chain_spec)
    }
}
