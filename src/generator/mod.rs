mod expression;
mod function;
mod program;
mod statement;

use crate::llvm_str;
use crate::parser::program::Program;
use crate::Result;
use llvm_sys::analysis::LLVMVerifierFailureAction;
use llvm_sys::prelude::{LLVMBuilderRef, LLVMContextRef, LLVMModuleRef, LLVMTypeRef, LLVMValueRef};
use llvm_sys::target_machine::{
    LLVMCodeGenFileType, LLVMCodeGenOptLevel, LLVMCodeModel, LLVMRelocMode, LLVMTargetRef,
};
use llvm_sys::{analysis, core, target, target_machine};
use log::{debug, info, trace, warn};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CStr;
use std::process::Command;

/// Generates LLVM IR based on the AST.
pub struct Generator {
    /// The root of the AST.
    program: Program,

    /// LLVM Context.
    context: LLVMContextRef,
    /// LLVM Module.
    module: LLVMModuleRef,
    /// LLVM Builder.
    builder: LLVMBuilderRef,

    /// LLVM variable map.
    local_vars: RefCell<HashMap<String, LLVMValueRef>>,
    /// Variables in the current scope
    scope_var_names: RefCell<Vec<Vec<String>>>,
}

impl Generator {
    /// Create a new generator from a [`Program`].
    ///
    /// [`Program`]: ../parser/program/struct.Program.html
    ///
    /// # Arguments
    /// * `program` - The root of the AST.
    /// * `name` - The name of the module to be created.
    pub fn new(program: Program, name: &str) -> Self {
        let context = unsafe { core::LLVMContextCreate() };
        Generator {
            program,
            context,
            module: unsafe { core::LLVMModuleCreateWithNameInContext(llvm_str!(name), context) },
            builder: unsafe { core::LLVMCreateBuilderInContext(context) },
            local_vars: RefCell::new(HashMap::new()),
            scope_var_names: RefCell::new(Vec::new()),
        }
    }

    /// Generate the LLVM IR from the module.
    pub fn generate(&self) -> Result<()> {
        self.gen_program(&self.program)?;
        debug!("Successfully generated IR");
        unsafe {
            analysis::LLVMVerifyModule(
                self.module,
                LLVMVerifierFailureAction::LLVMAbortProcessAction,
                ["".as_ptr() as *mut _].as_mut_ptr(), // TODO use error message
            )
        };
        debug!("Successfully verified module");
        Ok(())
    }

    /// Dump LLVM IR to stdout.
    pub fn generate_ir(&self, output: &str) {
        unsafe {
            core::LLVMPrintModuleToFile(
                self.module,
                llvm_str!(output),
                ["".as_ptr() as *mut _].as_mut_ptr(),
            );
        }
    }

    /// Generate an object file from the LLVM IR.
    ///
    /// # Arguments
    /// * `optimization` - Optimization level (0-3).
    /// * `output` - Output file path.
    pub fn generate_object_file(&self, optimization: u32, output: &str) {
        let target_triple = unsafe { target_machine::LLVMGetDefaultTargetTriple() };

        info!("Target: {}", unsafe {
            CStr::from_ptr(target_triple).to_str().unwrap()
        });

        unsafe {
            target::LLVM_InitializeAllTargetInfos();
            target::LLVM_InitializeAllTargets();
            target::LLVM_InitializeAllTargetMCs();
            target::LLVM_InitializeAllAsmParsers();
            target::LLVM_InitializeAllAsmPrinters();
        }
        trace!("Successfully initialized all LLVM targets");

        let mut target = std::mem::MaybeUninit::<LLVMTargetRef>::uninit();
        unsafe {
            target_machine::LLVMGetTargetFromTriple(
                target_triple,
                target.as_mut_ptr(),
                ["".as_ptr() as *mut _].as_mut_ptr(),
            );
            target.assume_init();
        }

        let optimization_level = match optimization {
            0 => LLVMCodeGenOptLevel::LLVMCodeGenLevelNone,
            1 => LLVMCodeGenOptLevel::LLVMCodeGenLevelLess,
            2 => LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault,
            3 => LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive,
            _ => {
                warn!("Invalid optimization level, defaulting to 2");
                LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault
            }
        };
        info!("Optimization level: {}", optimization);

        let target_machine = unsafe {
            target_machine::LLVMCreateTargetMachine(
                *target.as_ptr(),
                target_triple,
                llvm_str!(""),
                llvm_str!(""),
                optimization_level,
                LLVMRelocMode::LLVMRelocDefault, // TODO is this right?
                LLVMCodeModel::LLVMCodeModelDefault, // TODO is this right?
            )
        };
        trace!("Successfully created target machine");

        unsafe {
            target_machine::LLVMTargetMachineEmitToFile(
                target_machine,
                self.module,
                llvm_str!(output) as *mut _,
                LLVMCodeGenFileType::LLVMObjectFile,
                ["".as_ptr() as *mut _].as_mut_ptr(), // TODO use error message
            )
        };
        trace!("Successfully emitted to file");
    }

    /// Generates an executable from the object file by calling gcc.
    ///
    /// # Arguments
    /// * `object_file` - Path to the object file.
    /// * `output` - Path to the executable.
    pub fn generate_executable(&self, object_file: &str, output: &str) -> Result<()> {
        // TODO is there a better way to do this?
        match Command::new("gcc")
            .args(&[object_file, "-o", output])
            .output()
        {
            Ok(_) => {
                debug!("Successfully generated executable");
                Ok(())
            }
            Err(e) => Err(format!("Unable to link object file:\n{}", e)),
        }
    }

    /// Get LLVM i32 type in context.
    #[inline]
    fn i32_type(&self) -> LLVMTypeRef {
        unsafe { core::LLVMInt32TypeInContext(self.context) }
    }
}

impl Drop for Generator {
    fn drop(&mut self) {
        debug!("Cleaning up generator");
        unsafe {
            core::LLVMDisposeBuilder(self.builder);
            core::LLVMDisposeModule(self.module);
            core::LLVMContextDispose(self.context);
        }
    }
}

/// Convert a `&str` into `*const ::libc::c_char`
#[macro_export]
macro_rules! llvm_str {
    ($s:expr) => {
        format!("{}\0", $s).as_ptr() as *const i8
    };
}
