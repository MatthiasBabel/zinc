//!
//! The `std::convert::from_bits_field` function call.
//!

use franklin_crypto::bellman::pairing::ff::PrimeField;
use franklin_crypto::bellman::ConstraintSystem;
use franklin_crypto::circuit::num::AllocatedNum;

use zinc_bytecode::ScalarType;

use crate::core::execution_state::evaluation_stack::EvaluationStack;
use crate::error::RuntimeError;
use crate::gadgets::scalar::Scalar;
use crate::instructions::call_std::INativeCallable;
use crate::IEngine;

pub struct FieldFromBits;

impl<E: IEngine> INativeCallable<E> for FieldFromBits {
    fn call<CS: ConstraintSystem<E>>(
        &self,
        mut cs: CS,
        stack: &mut EvaluationStack<E>,
    ) -> Result<(), RuntimeError> {
        let mut bits = Vec::with_capacity(E::Fr::NUM_BITS as usize);
        for i in 0..E::Fr::NUM_BITS {
            let bit = stack.pop()?.try_into_value()?;
            let boolean = bit.to_boolean(cs.namespace(|| format!("to_boolean {}", i)))?;
            bits.push(boolean);
        }

        let num =
            AllocatedNum::pack_bits_to_element(cs.namespace(|| "pack_bits_to_element"), &bits)?;

        stack.push(
            Scalar::new_unchecked_variable(num.get_value(), num.get_variable(), ScalarType::Field)
                .into(),
        )?;

        Ok(())
    }
}