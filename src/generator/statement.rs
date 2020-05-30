use crate::Result;
use crate::generator::Generator;
use crate::llvm_str;
use crate::parser::statement::Statement;
use llvm_sys::core;
use log::{info, trace};

impl Generator {
    pub fn gen_statement(&self, statement: &Statement) -> Result<()> {
        trace!("Generating statement");
        match statement {
            Statement::CompoundStatement { statements } => {
                trace!("Generating compound statement");
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
                trace!("Generating if statement");
                unimplemented!()
            }

            Statement::ReturnStatement { value } => {
                trace!("Generating return statement");
                unsafe {
                    core::LLVMBuildRet(self.builder, self.gen_expression(value)?);
                }
                Ok(())
            }

            Statement::VariableDeclarationStatement { name, value } => {
                trace!("Generating variable declaration statement: {}", name);
                let var =
                    unsafe { core::LLVMBuildAlloca(self.builder, self.i32_type(), llvm_str!("")) };
                if name != "_" {
                    info!("Adding {} to local vars", name);
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
                trace!("Generating expression statement");
                unsafe {
                    let var = core::LLVMBuildAlloca(self.builder, self.i32_type(), llvm_str!(""));
                    core::LLVMBuildStore(self.builder, self.gen_expression(expression)?, var);
                }
                Ok(())
            }

            Statement::NoOpStatement => {
                trace!("Generating no op statement");
                Ok(())
            }
        }
    }
}
