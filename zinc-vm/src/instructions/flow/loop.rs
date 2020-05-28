use zinc_bytecode::LoopBegin;
use zinc_bytecode::LoopEnd;

use crate::core::VMInstruction;
use crate::core::VirtualMachine;
use crate::error::RuntimeError;

impl<VM: VirtualMachine> VMInstruction<VM> for LoopBegin {
    fn execute(&self, vm: &mut VM) -> Result<(), RuntimeError> {
        vm.loop_begin(self.iterations)
    }
}

impl<VM: VirtualMachine> VMInstruction<VM> for LoopEnd {
    fn execute(&self, vm: &mut VM) -> Result<(), RuntimeError> {
        vm.loop_end()
    }
}

#[cfg(test)]
mod test {
    use crate::tests::TestingError;
    use crate::tests::VMTestRunner;

    #[test]
    fn test_loop() -> Result<(), TestingError> {
        let _ = env_logger::builder().is_test(true).try_init();

        VMTestRunner::new()
            .add(zinc_bytecode::Push::new_field(0.into()))
            .add(zinc_bytecode::Store::new(0, 1))
            .add(zinc_bytecode::Push::new_field(0.into()))
            .add(zinc_bytecode::Store::new(1, 1))
            .add(zinc_bytecode::LoopBegin::new(10))
            .add(zinc_bytecode::Load::new(0, 1))
            .add(zinc_bytecode::Push::new_field(1.into()))
            .add(zinc_bytecode::Add)
            .add(zinc_bytecode::Store::new(0, 1))
            .add(zinc_bytecode::Load::new(0, 1))
            .add(zinc_bytecode::Load::new(1, 1))
            .add(zinc_bytecode::Add)
            .add(zinc_bytecode::Store::new(1, 1))
            .add(zinc_bytecode::LoopEnd)
            .add(zinc_bytecode::Load::new(0, 1))
            .add(zinc_bytecode::Load::new(1, 1))
            .test(&[55, 10])
    }
}
