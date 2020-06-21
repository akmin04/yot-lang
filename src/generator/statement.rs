use crate::c_str;
use crate::generator::Generator;
use crate::parser::statement::Statement;
use crate::Result;
use llvm_sys::core;
use log::{info, trace};

impl Generator {
    pub unsafe fn gen_statement(&self, statement: &Statement) -> Result<()> {
        trace!("Generating statement");
        match statement {
            Statement::CompoundStatement { statements } => {
                trace!("Generating compound statement");
                self.scope_var_names.borrow_mut().push(Vec::new());
                info!("Added new scope: #{}", self.scope_var_names.borrow().len());
                for statement in statements {
                    self.gen_statement(statement)?;
                }

                let mut local_vars_mut = self.local_vars.borrow_mut();
                for var in self.scope_var_names.borrow().last().unwrap() {
                    info!("Deleting variable `{}`", var);
                    local_vars_mut.remove(var);
                }

                self.scope_var_names.borrow_mut().pop();
                Ok(())
            }

            Statement::IfStatement { .. } => {
                trace!("Generating if statement");
                unimplemented!()
            }

            Statement::ReturnStatement { value } => {
                trace!("Generating return statement");
                core::LLVMBuildRet(self.builder, self.gen_expression(value)?);
                Ok(())
            }

            Statement::VariableDeclarationStatement { name, value } => {
                trace!("Generating variable declaration statement: {}", name);
                let mut local_vars_mut = self.local_vars.borrow_mut();

                if local_vars_mut.contains_key(name) {
                    return Err(format!("Variable `{}` already exists", name));
                }

                let var = core::LLVMBuildAlloca(self.builder, self.i32_type(), c_str!(""));
                if name != "_" {
                    info!("Adding `{}` to local vars", name);
                    local_vars_mut.insert(String::from(name), var);
                    self.scope_var_names
                        .borrow_mut()
                        .last_mut()
                        .unwrap()
                        .push(String::from(name));
                }

                drop(local_vars_mut);
                if let Some(value) = value {
                    core::LLVMBuildStore(self.builder, self.gen_expression(value)?, var);
                }
                Ok(())
            }

            Statement::ExpressionStatement { expression } => {
                trace!("Generating expression statement");
                self.gen_expression(expression)?;
                Ok(())
            }

            Statement::NoOpStatement => {
                trace!("Generating no op statement");
                Ok(())
            }
        }
    }
}
