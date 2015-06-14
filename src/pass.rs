use libc::c_uint;
use ffi::prelude::LLVMPassManagerRef;
use ffi::{core, target};
use ffi::transforms::pass_manager_builder as builder;
use ffi::transforms::pass_manager_builder::LLVMPassManagerBuilderRef;
use module::Module;
use target::TargetData;

/// Runs transformations on bitcode
pub struct PassManager {
    manager: LLVMPassManagerRef
}
native_ref!(PassManager, manager: LLVMPassManagerRef);
impl PassManager {
    /// Create a new pass manager
    pub fn new() -> PassManager {
        unsafe { core::LLVMCreatePassManager() }.into()
    }
    /// Adds target data information to a pass manage
    pub fn add_target_data(&self, data: &TargetData) {
        unsafe { target::LLVMAddTargetData(data.into(), self.into()) }
    }
    /// Initializes, executes on the provided module, and finalizes all of the passes scheduled in the pass manage
    pub fn run(&self, module: &Module) -> Result<(), ()> {
        if unsafe { core::LLVMRunPassManager(self.into(), module.into()) } == 0 {
            Ok(())
        } else {
            Err(())
        }
    }
    /// Pupulate this pass manager with the options given in the builder
    pub fn populate(&self, builder: PassManagerBuilder) {
        unsafe { builder::LLVMPassManagerBuilderPopulateModulePassManager(builder.into(), self.into()) }
    }
}
impl Drop for PassManager {
    fn drop(&mut self) {
        unsafe { core::LLVMDisposePassManager(self.into()) }
    }
}
/// Sets up a standard optimization sequence for languages like C and C++, allowing some APIs to customize the pass sequence in various ways
pub struct PassManagerBuilder {
    builder: LLVMPassManagerBuilderRef
}
native_ref!(PassManagerBuilder, builder: LLVMPassManagerBuilderRef);
impl PassManagerBuilder {
    /// Create a new pass manager builder
    pub fn new() -> PassManagerBuilder {
        unsafe { builder::LLVMPassManagerBuilderCreate().into() }
    }
    /// Set the optimisation level of the pass manager
    ///
    ///  0 = -O0, 1 = -O1, 2 = -O2, 3 = -O3
    pub fn with_opt_level(self, level: usize) -> PassManagerBuilder {
        unsafe { builder::LLVMPassManagerBuilderSetOptLevel((&self).into(), level as c_uint) };
        self
    }
    /// Specify how much we're optimizing for size
    ///
    ///  0 = none, 1 = -Os, 2 = -Oz
    pub fn with_size_level(self, size: usize) -> PassManagerBuilder {
        unsafe { builder::LLVMPassManagerBuilderSetOptLevel((&self).into(), size as c_uint) };
        self
    }
    /// Use an inliner with threshold given
    ///
    /// This threshold should be the degree to which the inlining should be done
    pub fn with_inliner(self, threshold: usize) -> PassManagerBuilder {
        unsafe { builder::LLVMPassManagerBuilderUseInlinerWithThreshold((&self).into(), threshold as c_uint) };
        self
    }
}
impl Drop for PassManagerBuilder {
    fn drop(&mut self) {
        unsafe { builder::LLVMPassManagerBuilderDispose(self.into()) }
    }
}
