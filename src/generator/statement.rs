use crate::generator::Generator;
use crate::llvm_str;
use crate::parser::statement::Statement;
use llvm_sys::core;
use log::{debug, info};

impl Generator {
    pub fn gen_statement(&self, statement: &Statement) -> Result<(), &'static str> {
        info!("Generating statement");
        match statement {
            Statement::CompoundStatement { statements } => {
                info!("Generating compound statement");
                for statement in statements {
                    self.gen_statement(statement)?;
                }
                Ok(())
            }

            Statement::IfStatement {
                condition: _,
                main_statement: _,
                else_statement: _,
            } => {
                info!("Generating if statement");
                unimplemented!()
            }

            Statement::ReturnStatement { value } => {
                info!("Generating return statement");
                unsafe {
                    core::LLVMBuildRet(self.builder, self.gen_expression(value)?);
                }
                Ok(())
            }

            Statement::VariableDeclarationStatement { name, value } => {
                info!("Generating variable declaration statement: {}", name);
                let var =
                    unsafe { core::LLVMBuildAlloca(self.builder, self.i32_type(), llvm_str!("")) };
                if name != "_" {
                    debug!("Adding {} to local vars", name);
                    let mut local_vars_mut = self.local_vars.borrow_mut();
                    local_vars_mut.insert(String::from(name), var);
                    drop(local_vars_mut);
                }

                if let Some(value) = value {
                    unsafe { core::LLVMBuildStore(self.builder, self.gen_expression(value)?, var) };
                }
                Ok(())
            }

            Statement::ExpressionStatement { expression } => {
                info!("Generating expression statement");
                unsafe {
                    let var = core::LLVMBuildAlloca(self.builder, self.i32_type(), llvm_str!(""));
                    core::LLVMBuildStore(self.builder, self.gen_expression(expression)?, var);
                }
                Ok(())
            }

            Statement::NoOpStatement => {
                info!("Generating no op statement");
                Ok(())
            }
        }
    }
}
