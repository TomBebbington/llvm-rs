use libc::c_uint;
use ffi::prelude::LLVMPassManagerRef;
use ffi::core;
use ffi::transforms::pass_manager_builder as builder;
use ffi::transforms::pass_manager_builder::LLVMPassManagerBuilderRef;
use module::Module;

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
    /// Run this pass manager
    pub fn run(&self, module: &Module) -> Result<(), ()> {
        if unsafe { core::LLVMRunPassManager(self.into(), module.into()) } == 0 {
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn populate(&self, builder: PassManagerBuilder) {
        unsafe { builder::LLVMPassManagerBuilderPopulateModulePassManager(builder.into(), self.into()) }
    }
}
impl Drop for PassManager {
    fn drop(&mut self) {
        unsafe { core::LLVMDisposePassManager(self.into()) }
    }
}
pub struct PassManagerBuilder {
    builder: LLVMPassManagerBuilderRef
}
native_ref!(PassManagerBuilder, builder: LLVMPassManagerBuilderRef);
impl PassManagerBuilder {
    /// Set the optimisation level of the pass manager
    pub fn set_opt_level(&self, level: usize) {
        unsafe { builder::LLVMPassManagerBuilderSetOptLevel(self.into(), level as c_uint) }
    }
    /// Set the size level of the pass manager
    pub fn set_size_level(&self, size: usize) {
        unsafe { builder::LLVMPassManagerBuilderSetOptLevel(self.into(), size as c_uint) }
    }
}
impl Drop for PassManagerBuilder {
    fn drop(&mut self) {
        unsafe { builder::LLVMPassManagerBuilderDispose(self.into()) }
    }
}
