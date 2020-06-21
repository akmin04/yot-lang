use crate::c_str;
use crate::generator::Generator;
use crate::parser::function::Function;
use crate::Result;
use llvm_sys::core;
use log::{info, trace};

impl Generator {
    pub unsafe fn gen_function(&self, function: &Function) -> Result<()> {
        trace!("Generating function");

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
        let llvm_function = core::LLVMAddFunction(
            self.module,
            c_str!(name),
            core::LLVMFunctionType(
                self.i32_type(),
                arg_types.as_mut_ptr(),
                args.len() as u32,
                0,
            ),
        );

        if let Function::RegularFunction {
            name: _,
            args: _,
            statement,
        } = function
        {
            // Append empty block
            let entry =
                core::LLVMAppendBasicBlockInContext(self.context, llvm_function, c_str!("entry"));

            core::LLVMPositionBuilderAtEnd(self.builder, entry);

            for (i, arg_name) in args.iter().enumerate() {
                // Set arg name in function prototype
                let arg = core::LLVMGetParam(llvm_function, i as u32);
                core::LLVMSetValueName2(arg, c_str!(arg_name), arg_name.len());

                let mut local_vars_mut = self.local_vars.borrow_mut();

                let var = core::LLVMBuildAlloca(self.builder, self.i32_type(), c_str!(""));
                if arg_name != "_" {
                    info!("Adding `{}` to local vars", arg_name);
                    local_vars_mut.insert(String::from(arg_name), var);
                }

                core::LLVMBuildStore(self.builder, arg, var);
            }

            // Generate function statement
            self.gen_statement(&statement)?;
        }

        Ok(())
    }
}
