use crate::element::{Element, ElementOperator};
use crate::RuntimeError;
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use std::fmt::{Debug, Display, Formatter, Error};

#[derive(Debug, Clone)]
pub struct PrimitiveElement {
    value: u128,
}

impl Display for PrimitiveElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        Display::fmt(&self.value, f)
    }
}

impl Element for PrimitiveElement {}

pub struct PrimitiveElementOperator {}

impl ElementOperator<PrimitiveElement> for PrimitiveElementOperator {
    fn constant_u64(&mut self, value: u64) -> Result<PrimitiveElement, RuntimeError> {
        Ok(PrimitiveElement {value: value as u128})
    }

    fn constant_bigint(&mut self, value: &BigInt) -> Result<PrimitiveElement, RuntimeError> {
        let v = value.to_u128().ok_or(RuntimeError::IntegerOverflow)?;
        Ok(PrimitiveElement { value: v })
    }

    fn add(&mut self, left: PrimitiveElement, right: PrimitiveElement) -> Result<PrimitiveElement, RuntimeError> {
        let value = left.value.checked_add(right.value).ok_or(RuntimeError::IntegerOverflow)?;
        Ok(PrimitiveElement { value })
    }

    fn sub(&mut self, left: PrimitiveElement, right: PrimitiveElement) -> Result<PrimitiveElement, RuntimeError> {
        let value = left.value.checked_add(right.value).ok_or(RuntimeError::IntegerOverflow)?;
        Ok(PrimitiveElement { value })
    }

    fn mul(&mut self, left: PrimitiveElement, right: PrimitiveElement) -> Result<PrimitiveElement, RuntimeError> {
        let value = left.value.checked_mul(right.value).ok_or(RuntimeError::IntegerOverflow)?;
        Ok(PrimitiveElement { value })

    }

    fn div_rem(&mut self, left: PrimitiveElement, right: PrimitiveElement)
        -> Result<(PrimitiveElement, PrimitiveElement), RuntimeError>
    {
        let div = left.value.checked_div(right.value).ok_or(RuntimeError::IntegerOverflow)?;
        let rem = left.value.checked_rem(right.value).ok_or(RuntimeError::IntegerOverflow)?;

        Ok((
           PrimitiveElement { value: div },
           PrimitiveElement { value: rem },
        ))
    }
}
