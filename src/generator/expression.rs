use crate::generator::Generator;
use crate::lexer::tokens::Literal;
use crate::llvm_str;
use crate::parser::expression::Expression;
use llvm_sys::core;
use llvm_sys::prelude::LLVMValueRef;
use llvm_sys::LLVMIntPredicate;
use log::{debug, error, info};
use std::process;

type MaybeLLVMValueRef = Result<LLVMValueRef, &'static str>;

impl Generator {
    pub fn gen_expression(&self, expression: &Expression) -> MaybeLLVMValueRef {
        info!("Generating expression");
        match expression {
            Expression::LiteralExpression { value } => {
                info!("Generating literal expression: {:?}", value);
                match value {
                    Literal::Integer(i) => unsafe {
                        debug!("Integer literal: {}", i);
                        Ok(core::LLVMConstInt(self.i32_type(), *i as u64, false as i32))
                    },
                    Literal::Str(s) => unsafe {
                        debug!("Str literal: {}", s);
                        Ok(core::LLVMConstString(
                            llvm_str!(s),
                            s.len() as u32,
                            false as i32,
                        ))
                    },
                }
            }

            Expression::ParenExpression { expression } => {
                info!("Generating paren expression");
                self.gen_expression(expression)
            }

            Expression::VariableReferenceExpression { name } => {
                info!("Generating variable reference expression: {}", name);
                if let Some(var) = self.local_vars.borrow().get(name) {
                    debug!("Local variable: {}", name);
                    unsafe {
                        return Ok(core::LLVMBuildLoad2(
                            self.builder,
                            self.i32_type(),
                            *var,
                            llvm_str!(""),
                        ));
                    }
                } else if let Some(i) = self.fn_args.borrow().iter().position(|a| a == name) {
                    debug!("Function param");
                    unsafe {
                        return Ok(core::LLVMGetParam(
                            core::LLVMGetLastFunction(self.module),
                            i as u32,
                        ));
                    }
                }
                Err("Unresolved variable reference")
            }

            Expression::FunctionCallExpression { name, args } => {
                info!("Generating function call expression: {}", name);
                let mut llvm_args: Vec<LLVMValueRef> = Vec::new();
                for arg in args {
                    llvm_args.push(self.gen_expression(arg)?);
                }

                unsafe {
                    Ok(core::LLVMBuildCall(
                        self.builder,
                        core::LLVMGetNamedFunction(self.module, llvm_str!(name)),
                        llvm_args.as_mut_ptr(),
                        args.len() as u32,
                        llvm_str!(""),
                    ))
                }
            }

            Expression::BinaryExpression {
                op,
                l_expression,
                r_expression,
            } => {
                info!("Generating binary expression");
                let l = self.gen_expression(l_expression)?;
                let r = self.gen_expression(r_expression)?;

                // unsafe {
                match &op[..] {
                    "+" => unsafe { Ok(core::LLVMBuildAdd(self.builder, l, r, llvm_str!(""))) },
                    "-" => unsafe { Ok(core::LLVMBuildSub(self.builder, l, r, llvm_str!(""))) },
                    "*" => unsafe { Ok(core::LLVMBuildMul(self.builder, l, r, llvm_str!(""))) },
                    "/" => unsafe { Ok(core::LLVMBuildSDiv(self.builder, l, r, llvm_str!(""))) },
                    "==" | "!=" | "<" | ">" | "<=" | ">=" => {
                        let cmp = unsafe {
                            core::LLVMBuildICmp(
                                self.builder,
                                match &op[..] {
                                    "==" => LLVMIntPredicate::LLVMIntEQ,
                                    "!=" => LLVMIntPredicate::LLVMIntNE,
                                    "<" => LLVMIntPredicate::LLVMIntSLT,
                                    ">" => LLVMIntPredicate::LLVMIntSGT,
                                    "<=" => LLVMIntPredicate::LLVMIntSLE,
                                    ">=" => LLVMIntPredicate::LLVMIntSGE,
                                    _ => {
                                        error!("Generation Unhandled comparison binary operation");
                                        process::exit(1);
                                    }
                                },
                                l,
                                r,
                                llvm_str!(""),
                            )
                        };
                        // Cast i1 to i32
                        let cmp_i32 = unsafe {
                            core::LLVMBuildZExt(self.builder, cmp, self.i32_type(), llvm_str!(""))
                        };
                        Ok(cmp_i32)
                    }
                    "=" => unimplemented!(),
                    _ => {
                        error!("Generation: Misidentified binary expression");
                        process::exit(1);
                    }
                }
                // }
            }

            Expression::UnaryExpression { op, expression } => {
                info!("Generating unary expression");
                match &op[..] {
                    "-" => unsafe {
                        Ok(core::LLVMBuildNeg(
                            self.builder,
                            self.gen_expression(expression)?,
                            llvm_str!(""),
                        ))
                    },
                    _ => {
                        error!("Generation: Misidentified unary expression");
                        process::exit(1);
                    }
                }
            }
        }
    }
}
