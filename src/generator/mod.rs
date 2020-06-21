mod expression;
mod function;
mod program;
mod statement;

use crate::c_str;
use crate::parser::program::Program;
use crate::Result;
use libc::c_char;
use llvm_sys::analysis::LLVMVerifierFailureAction;
use llvm_sys::prelude::{LLVMBuilderRef, LLVMContextRef, LLVMModuleRef, LLVMTypeRef, LLVMValueRef};
use llvm_sys::target_machine::{
    LLVMCodeGenFileType, LLVMCodeGenOptLevel, LLVMCodeModel, LLVMRelocMode, LLVMTarget,
};
use llvm_sys::{analysis, core, target, target_machine};
use log::{debug, error, info, trace, warn};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CStr;
use std::process::Command;
use std::ptr;

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
    pub unsafe fn new(program: Program, name: &str) -> Self {
        let context = core::LLVMContextCreate();
        Generator {
            program,
            context,
            module: core::LLVMModuleCreateWithNameInContext(c_str!(name), context),
            builder: core::LLVMCreateBuilderInContext(context),
            local_vars: RefCell::new(HashMap::new()),
            scope_var_names: RefCell::new(Vec::new()),
        }
    }

    /// Generate the LLVM IR from the module.
    pub unsafe fn generate(&self) -> Result<()> {
        self.gen_program(&self.program)?;
        debug!("Successfully generated program");
        Ok(())
    }

    /// Verify LLVM IR.
    pub unsafe fn verify(&self) -> Result<()> {
        let mut error = ptr::null_mut::<c_char>();
        analysis::LLVMVerifyModule(
            self.module,
            LLVMVerifierFailureAction::LLVMReturnStatusAction,
            &mut error,
        );
        if !error.is_null() {
            let error = CStr::from_ptr(error).to_str().unwrap().to_string();
            if !error.is_empty() {
                return Err(error);
            }
        }
        debug!("Successfully verified module");
        Ok(())
    }

    /// Dump LLVM IR to stdout.
    pub unsafe fn generate_ir(&self, output: &str) -> Result<()> {
        let mut error = ptr::null_mut::<c_char>();
        core::LLVMPrintModuleToFile(self.module, c_str!(output), &mut error);
        if !error.is_null() {
            let error = CStr::from_ptr(error).to_str().unwrap().to_string();
            if !error.is_empty() {
                return Err(error);
            }
        }
        Ok(())
    }

    /// Generate an object file from the LLVM IR.
    ///
    /// # Arguments
    /// * `optimization` - Optimization level (0-3).
    /// * `output` - Output file path.
    pub unsafe fn generate_object_file(&self, optimization: u32, output: &str) -> Result<()> {
        let target_triple = target_machine::LLVMGetDefaultTargetTriple();

        info!(
            "Target: {}",
            CStr::from_ptr(target_triple).to_str().unwrap()
        );

        target::LLVM_InitializeAllTargetInfos();
        target::LLVM_InitializeAllTargets();
        target::LLVM_InitializeAllTargetMCs();
        target::LLVM_InitializeAllAsmParsers();
        target::LLVM_InitializeAllAsmPrinters();
        trace!("Successfully initialized all LLVM targets");

        let mut target = ptr::null_mut::<LLVMTarget>();
        let mut error = ptr::null_mut::<c_char>();
        target_machine::LLVMGetTargetFromTriple(target_triple, &mut target, &mut error);
        if !error.is_null() {
            let error = CStr::from_ptr(error).to_str().unwrap().to_string();
            if !error.is_empty() {
                return Err(error);
            }
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

        let target_machine = target_machine::LLVMCreateTargetMachine(
            target,
            target_triple,
            c_str!("generic"),
            c_str!(""),
            optimization_level,
            LLVMRelocMode::LLVMRelocDefault, // TODO is this right?
            LLVMCodeModel::LLVMCodeModelDefault, // TODO is this right?
        );
        trace!("Successfully created target machine");

        let mut target = ptr::null_mut::<c_char>();
        target_machine::LLVMTargetMachineEmitToFile(
            target_machine,
            self.module,
            c_str!(output) as *mut _,
            LLVMCodeGenFileType::LLVMObjectFile,
            &mut target,
        );
        if !target.is_null() {
            let error = CStr::from_ptr(error).to_str().unwrap();
            error!("{}", error);
        };
        trace!("Successfully emitted to file");
        Ok(())
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
            .spawn()
        {
            Ok(_) => {
                debug!("Successfully generated executable: {}", output);
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

/// Convert a `&str` into `*const libc::c_char`
#[macro_export]
macro_rules! c_str {
    ($s:expr) => {
        format!("{}\0", $s).as_ptr() as *const libc::c_char
    };
}
