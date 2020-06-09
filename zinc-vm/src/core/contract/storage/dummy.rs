use num_bigint::BigInt;
use num_traits::ToPrimitive;

use ff::PrimeField;
use ff::PrimeFieldRepr;

use zinc_bytecode::DataType;

use crate::core::contract::storage::sha256;
use crate::error::RuntimeError;
use crate::gadgets::contract::merkle_tree::leaf::Leaf as MerkleTreeLeaf;
use crate::gadgets::contract::merkle_tree::IMerkleTree;
use crate::gadgets::scalar::Scalar;
use crate::IEngine;

pub struct Storage<E: IEngine> {
    depth: usize,
    tree: Vec<Vec<u8>>,
    leaf_values: Vec<Vec<Option<Scalar<E>>>>,
}

impl<E: IEngine> Storage<E> {
    pub fn new(fields: Vec<DataType>) -> Self {
        let depth = (fields.len() as f64).log2().ceil() as usize;

        let mut result = Self {
            depth,
            tree: vec![vec![]; 1 << (depth + 1)],
            leaf_values: vec![vec![]; 1 << depth],
        };

        for (index, field) in fields.into_iter().enumerate() {
            let values = field
                .to_scalar_types()
                .into_iter()
                .map(|r#type| Some(Scalar::<E>::new_constant_int(0, r#type)))
                .collect();
            result.leaf_values[index] = values;
        }
        result.rebuild_tree();

        result
    }

    fn rebuild_tree(&mut self) {
        for i in (1..(1 << (self.depth + 1))).rev() {
            if i < (1 << self.depth) {
                self.tree[i] =
                    sha256::sha256_of_concat::<E>(&self.tree[i * 2], &self.tree[i * 2 + 1]);
            } else {
                self.tree[i] =
                    sha256::leaf_value_hash::<E>(self.leaf_values[i - (1 << self.depth)].clone());
            }
        }
    }
}

impl<E: IEngine> IMerkleTree<E> for Storage<E> {
    fn depth(&self) -> usize {
        self.depth
    }

    fn root_hash(&self) -> Option<E::Fr> {
        let mut hash_as_buf = self.tree[1].clone();

        hash_as_buf
            .truncate(zinc_const::BITLENGTH_SHA256_HASH_TRUNCATED / zinc_const::BITLENGTH_BYTE);
        hash_as_buf.resize(
            zinc_const::BITLENGTH_SHA256_HASH / zinc_const::BITLENGTH_BYTE,
            0,
        );

        let mut hash_le = vec![];
        for i in &hash_as_buf {
            let mut current_byte: u8 = 0;
            for j in 0..zinc_const::BITLENGTH_BYTE {
                current_byte <<= 1;
                current_byte += (i >> j) & 1u8;
            }
            hash_le.push(current_byte);
        }

        let mut hash_repr = <E::Fr as PrimeField>::Repr::default();
        hash_repr.read_le(hash_le.as_slice()).unwrap();
        E::Fr::from_repr(hash_repr).ok()
    }

    fn load(&self, index: BigInt) -> Result<MerkleTreeLeaf<E>, RuntimeError> {
        let index = index.to_usize().ok_or(RuntimeError::ExpectedUsize(index))?;

        let mut result = MerkleTreeLeaf::new(self.leaf_values[index].as_slice(), None);

        let mut current_node = 1;
        for i in (0..self.depth).rev() {
            let next = current_node * 2 + ((index >> i) & 1usize);
            let mut current_auth_path_node_hash = vec![];
            for i in &self.tree[next ^ 1usize] {
                for j in (0..zinc_const::BITLENGTH_BYTE).rev() {
                    current_auth_path_node_hash.push(Some(((i >> j) & 1u8) == 1u8));
                }
            }
            result.authentication_path.push(current_auth_path_node_hash);

            current_node = next;
        }

        result.authentication_path.reverse();

        Ok(result)
    }

    fn store(
        &mut self,
        index: BigInt,
        values: Vec<Option<Scalar<E>>>,
    ) -> Result<MerkleTreeLeaf<E>, RuntimeError> {
        let index = index.to_usize().ok_or(RuntimeError::ExpectedUsize(index))?;

        let mut result = MerkleTreeLeaf::new(self.leaf_values[index].as_slice(), None);

        let mut current_node = 1;
        for i in (0..self.depth).rev() {
            let next = current_node * 2 + ((index >> i) & 1usize);
            let mut current_auth_path_node_hash = vec![];
            for i in &self.tree[next ^ 1usize] {
                for j in (0..zinc_const::BITLENGTH_BYTE).rev() {
                    current_auth_path_node_hash.push(Some(((i >> j) & 1u8) == 1u8));
                }
            }
            result.authentication_path.push(current_auth_path_node_hash);

            current_node = next;
        }

        result.authentication_path.reverse();

        self.leaf_values[index] = values;
        self.rebuild_tree();

        Ok(result)
    }
}
