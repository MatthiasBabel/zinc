use franklin_crypto::bellman::ConstraintSystem;

use zinc_bytecode::Le;

use crate::core::execution_state::cell::Cell;
use crate::core::virtual_machine::IVirtualMachine;
use crate::error::RuntimeError;
use crate::gadgets;
use crate::instructions::IExecutable;

impl<VM: IVirtualMachine> IExecutable<VM> for Le {
    fn execute(&self, vm: &mut VM) -> Result<(), RuntimeError> {
        let right = vm.pop()?.try_into_value()?;
        let left = vm.pop()?.try_into_value()?;

        let cs = vm.constraint_system();
        let le = gadgets::comparison::lesser_or_equals(cs.namespace(|| "le"), &left, &right)?;

        vm.push(Cell::Value(le))
    }
}

#[cfg(test)]
mod test {
    use crate::tests::TestRunner;
    use crate::tests::TestingError;

    use zinc_bytecode::IntegerType;

    #[test]
    fn test_le() -> Result<(), TestingError> {
        let _ = env_logger::builder().is_test(true).try_init();

        TestRunner::new()
            .add(zinc_bytecode::Push::new(2.into(), IntegerType::I8.into()))
            .add(zinc_bytecode::Push::new(1.into(), IntegerType::I8.into()))
            .add(zinc_bytecode::Le)
            .add(zinc_bytecode::Push::new(2.into(), IntegerType::I8.into()))
            .add(zinc_bytecode::Push::new(2.into(), IntegerType::I8.into()))
            .add(zinc_bytecode::Le)
            .add(zinc_bytecode::Push::new(1.into(), IntegerType::I8.into()))
            .add(zinc_bytecode::Push::new(2.into(), IntegerType::I8.into()))
            .add(zinc_bytecode::Le)
            .add(zinc_bytecode::Push::new(
                (-2).into(),
                IntegerType::I8.into(),
            ))
            .add(zinc_bytecode::Push::new(2.into(), IntegerType::I8.into()))
            .add(zinc_bytecode::Le)
            .add(zinc_bytecode::Push::new(2.into(), IntegerType::I8.into()))
            .add(zinc_bytecode::Push::new(
                (-2).into(),
                IntegerType::I8.into(),
            ))
            .add(zinc_bytecode::Le)
            .test(&[0, 1, 1, 1, 0])
    }
}
