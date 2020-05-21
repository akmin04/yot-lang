use crate::generator::Generator;
use crate::llvm_str;
use crate::parser::function::Function;
use llvm_sys::analysis;
use llvm_sys::analysis::LLVMVerifierFailureAction;
use llvm_sys::core;
use log::{debug, info};

impl Generator {
    pub fn gen_function(&self, function: &Function) -> Result<(), &'static str> {
        info!("Generating function");

        let args = match function {
            Function::RegularFunction {
                name: _,
                args,
                statement: _,
            } => args,
            Function::ExternalFunction { name: _, args } => args,
        };

        let name = match function {
            Function::RegularFunction {
                name,
                args: _,
                statement: _,
            } => name,
            Function::ExternalFunction { name, args: _ } => name,
        };
        // All args are i32 for now
        let mut arg_types = vec![self.i32_type(); args.len()];

        // Create function
        let llvm_function = unsafe {
            core::LLVMAddFunction(
                self.module,
                llvm_str!(name),
                core::LLVMFunctionType(
                    self.i32_type(),
                    arg_types.as_mut_ptr(),
                    args.len() as u32,
                    0,
                ),
            )
        };

        if let Function::RegularFunction {
            name: _,
            args: _,
            statement,
        } = function
        {
            // Name and create parameters
            let mut fn_args_mut = self.fn_args.borrow_mut();

            for (i, arg_name) in args.iter().enumerate() {
                let arg = unsafe { core::LLVMGetParam(llvm_function, i as u32) };
                fn_args_mut.push(String::from(arg_name));

                unsafe { core::LLVMSetValueName2(arg, llvm_str!(arg_name), arg_name.len()) };
            }
            drop(fn_args_mut);
            // Append empty block
            let entry = unsafe {
                core::LLVMAppendBasicBlockInContext(self.context, llvm_function, llvm_str!("entry"))
            };

            unsafe { core::LLVMPositionBuilderAtEnd(self.builder, entry) };

            // Generate function statement
            self.gen_statement(&statement)?;
        }

        unsafe {
            analysis::LLVMVerifyFunction(
                llvm_function,
                LLVMVerifierFailureAction::LLVMAbortProcessAction,
            );
        }
        debug!("Successfully verified function");
        Ok(())
    }
}
