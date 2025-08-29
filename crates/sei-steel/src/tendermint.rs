use serde::{Deserialize, Serialize};
use tendermint::merkle::Proof;

/// A commitment to a field of the cosmos block at a specific index in a Merkle tree, along with the slot of the block
#[derive(Clone, Serialize, Deserialize)]
pub struct TendermintCommitment {
    slot: u64,
    proof: Proof,
}
