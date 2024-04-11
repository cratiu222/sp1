use crate::stark::PROOF_MAX_NUM_PVS;

use super::Word;
use core::fmt::Debug;
use itertools::Itertools;
use p3_field::{AbstractField, PrimeField32};
use serde::{Deserialize, Serialize};
use std::iter::once;

pub const PV_DIGEST_NUM_WORDS: usize = 8;

/// The PublicValues struct is used to store all of a shard proof's public values.
#[derive(Serialize, Deserialize, Clone, Copy, Default, Debug)]
pub struct PublicValues<W, T> {
    /// The hash of all the bytes that the guest program has written to public values.
    pub committed_value_digest: [W; PV_DIGEST_NUM_WORDS],

    /// The shard number.
    pub shard: T,

    /// The shard's start program counter.
    pub start_pc: T,

    /// The expected start program counter for the next shard.
    pub next_pc: T,

    /// The exit code of the program.  Only valid if halt has been executed.
    pub exit_code: T,
}

impl PublicValues<u32, u32> {
    /// Convert the public values into a vector of field elements.  This function will pad the vector
    /// to the maximum number of public values.
    pub fn to_vec<F: AbstractField>(&self) -> Vec<F> {
        let mut ret = self
            .committed_value_digest
            .iter()
            .flat_map(|w| Word::<F>::from(*w).into_iter())
            .chain(once(F::from_canonical_u32(self.shard)))
            .chain(once(F::from_canonical_u32(self.start_pc)))
            .chain(once(F::from_canonical_u32(self.next_pc)))
            .chain(once(F::from_canonical_u32(self.exit_code)))
            .collect_vec();

        assert!(
            ret.len() <= PROOF_MAX_NUM_PVS,
            "Too many public values: {}",
            ret.len()
        );

        ret.resize(PROOF_MAX_NUM_PVS, F::zero());

        ret
    }
}

impl<F: AbstractField> PublicValues<Word<F>, F> {
    /// Convert a vector of field elements into a PublicValues struct.
    pub fn from_vec(data: Vec<F>) -> Self {
        let mut iter = data.iter().cloned();

        let mut committed_value_digest = Vec::new();
        for _ in 0..PV_DIGEST_NUM_WORDS {
            committed_value_digest.push(Word::from_iter(&mut iter));
        }

        // Collecting the remaining items into a tuple.  Note that it is only getting the first
        // four items, as the rest would be padded values.
        let remaining_items = iter.collect_vec();
        if remaining_items.len() < 4 {
            panic!("Invalid number of items in the serialized vector.");
        }

        let [shard, start_pc, next_pc, exit_code] = match &remaining_items.as_slice()[0..4] {
            [shard, start_pc, next_pc, exit_code] => [shard, start_pc, next_pc, exit_code],
            _ => unreachable!(),
        };

        Self {
            committed_value_digest: committed_value_digest.try_into().unwrap(),
            shard: shard.to_owned(),
            start_pc: start_pc.to_owned(),
            next_pc: next_pc.to_owned(),
            exit_code: exit_code.to_owned(),
        }
    }
}

impl<F: PrimeField32> PublicValues<Word<F>, F> {
    /// Returns the commit digest as a vector of little-endian bytes.
    pub fn commit_digest_bytes(&self) -> Vec<u8> {
        self.committed_value_digest
            .iter()
            .flat_map(|w| w.into_iter().map(|f| f.as_canonical_u32() as u8))
            .collect_vec()
    }
}

#[cfg(test)]
mod tests {
    use crate::air::public_values;

    /// Check that the PI_DIGEST_NUM_WORDS number match the zkVM crate's.
    #[test]
    fn test_public_values_digest_num_words_consistency_zkvm() {
        assert_eq!(
            public_values::PV_DIGEST_NUM_WORDS,
            sp1_zkvm::PV_DIGEST_NUM_WORDS
        );
    }
}