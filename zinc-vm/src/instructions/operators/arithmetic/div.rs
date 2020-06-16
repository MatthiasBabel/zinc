//!
//! The `Div` instruction.
//!

use franklin_crypto::bellman::ConstraintSystem;

use zinc_bytecode::Div;
use zinc_bytecode::ScalarType;

use crate::core::execution_state::cell::Cell;
use crate::core::virtual_machine::IVirtualMachine;
use crate::error::RuntimeError;
use crate::gadgets;
use crate::gadgets::scalar::expectation::ITypeExpectation;
use crate::gadgets::scalar::Scalar;
use crate::instructions::IExecutable;

impl<VM: IVirtualMachine> IExecutable<VM> for Div {
    fn execute(self, vm: &mut VM) -> Result<(), RuntimeError> {
        let right = vm.pop()?.try_into_value()?;
        let left = vm.pop()?.try_into_value()?;

        let condition = vm.condition_top()?;
        let scalar_type = ScalarType::expect_same(left.get_type(), right.get_type())?;

        let cs = vm.constraint_system();

        let div = match scalar_type {
            ScalarType::Field => {
                let one = Scalar::new_constant_usize(1, right.get_type());
                let denom = gadgets::select::conditional(
                    cs.namespace(|| "select denom"),
                    &condition,
                    &right,
                    &one,
                )?;
                let inverse =
                    gadgets::arithmetic::field::inverse(cs.namespace(|| "inverse"), &denom)?;
                gadgets::arithmetic::mul::mul(cs.namespace(|| "div"), &left, &inverse)?
            }
            ScalarType::Integer(_) => {
                let (unchecked_div, _rem) = gadgets::arithmetic::div_rem::div_rem_conditional(
                    cs.namespace(|| "div_rem_conditional"),
                    &condition,
                    &left,
                    &right,
                )?;

                Scalar::conditional_type_check(
                    cs.namespace(|| "type check"),
                    &condition,
                    &unchecked_div,
                    scalar_type,
                )?
            }
            _ => {
                return Err(RuntimeError::TypeError {
                    expected: "integer or field".to_string(),
                    actual: scalar_type.to_string(),
                })
            }
        };

        vm.push(Cell::Value(div))
    }
}

#[cfg(test)]
mod test {
    use crate::tests::TestRunner;
    use crate::tests::TestingError;

    use zinc_bytecode::IntegerType;

    #[test]
    fn test_div() -> Result<(), TestingError> {
        TestRunner::new()
            .push(zinc_bytecode::Push::new((9).into(), IntegerType::I8.into()))
            .push(zinc_bytecode::Push::new((4).into(), IntegerType::I8.into()))
            .push(zinc_bytecode::Div)
            .push(zinc_bytecode::Push::new((9).into(), IntegerType::I8.into()))
            .push(zinc_bytecode::Push::new(
                (-4).into(),
                IntegerType::I8.into(),
            ))
            .push(zinc_bytecode::Div)
            .push(zinc_bytecode::Push::new(
                (-9).into(),
                IntegerType::I8.into(),
            ))
            .push(zinc_bytecode::Push::new((4).into(), IntegerType::I8.into()))
            .push(zinc_bytecode::Div)
            .push(zinc_bytecode::Push::new(
                (-9).into(),
                IntegerType::I8.into(),
            ))
            .push(zinc_bytecode::Push::new(
                (-4).into(),
                IntegerType::I8.into(),
            ))
            .push(zinc_bytecode::Div)
            .test(&[3, -3, -2, 2])
    }
}