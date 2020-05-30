use crate::generator::Generator;
use crate::lexer::tokens::Literal;
use crate::llvm_str;
use crate::parser::expression::Expression;
use crate::Result;
use llvm_sys::core;
use llvm_sys::prelude::LLVMValueRef;
use llvm_sys::LLVMIntPredicate;
use log::trace;

impl Generator {
    pub fn gen_expression(&self, expression: &Expression) -> Result<LLVMValueRef> {
        trace!("Generating expression");
        match expression {
            Expression::LiteralExpression { value } => {
                trace!("Generating literal expression: {:?}", value);
                match value {
                    Literal::Integer(i) => unsafe {
                        trace!("Integer literal: {}", i);
                        Ok(core::LLVMConstInt(self.i32_type(), *i as u64, false as i32))
                    },
                    Literal::Str(s) => unsafe {
                        trace!("Str literal: {}", s);
                        Ok(core::LLVMConstString(
                            llvm_str!(s),
                            s.len() as u32,
                            false as i32,
                        ))
                    },
                }
            }

            Expression::ParenExpression { expression } => {
                trace!("Generating paren expression");
                self.gen_expression(expression)
            }

            Expression::VariableReferenceExpression { name } => {
                trace!("Generating variable reference expression: {}", name);
                if let Some(var) = self.local_vars.borrow().get(name) {
                    trace!("Local variable: {}", name);
                    unsafe {
                        return Ok(core::LLVMBuildLoad2(
                            self.builder,
                            self.i32_type(),
                            *var,
                            llvm_str!(""),
                        ));
                    }
                } else if let Some(i) = self.fn_args.borrow().iter().position(|a| a == name) {
                    trace!("Function param");
                    unsafe {
                        return Ok(core::LLVMGetParam(
                            core::LLVMGetLastFunction(self.module),
                            i as u32,
                        ));
                    }
                }
                Err(format!("Unresolved variable reference `{}`", name))
            }

            Expression::FunctionCallExpression { name, args } => {
                trace!("Generating function call expression: {}", name);
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
                trace!("Generating binary expression");

                let r = self.gen_expression(r_expression)?;

                if op == "=" {
                    if let Expression::VariableReferenceExpression { name } = l_expression.as_ref()
                    {
                        let local_vars_immut = self.local_vars.borrow();
                        let var = match local_vars_immut.get(name) {
                            Some(v) => v,
                            None => {
                                return Err(format!(
                                    "Tried to assign to undefined variable `{}`",
                                    name
                                ))
                            }
                        };

                        unsafe {
                            core::LLVMBuildStore(self.builder, r, *var);
                        }

                        Ok(r)
                    } else {
                        return Err(format!("Expected variable reference on assignment"));
                    }
                } else {
                    let l = self.gen_expression(l_expression)?;

                    match &op[..] {
                        "+" => unsafe { Ok(core::LLVMBuildAdd(self.builder, l, r, llvm_str!(""))) },
                        "-" => unsafe { Ok(core::LLVMBuildSub(self.builder, l, r, llvm_str!(""))) },
                        "*" => unsafe { Ok(core::LLVMBuildMul(self.builder, l, r, llvm_str!(""))) },
                        "/" => unsafe {
                            Ok(core::LLVMBuildSDiv(self.builder, l, r, llvm_str!("")))
                        },
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
                                            return Err(format!(
                                                "Unhandled comparison binary operation `{}`",
                                                op
                                            ))
                                        }
                                    },
                                    l,
                                    r,
                                    llvm_str!(""),
                                )
                            };
                            // Cast i1 to i32
                            let cmp_i32 = unsafe {
                                core::LLVMBuildZExt(
                                    self.builder,
                                    cmp,
                                    self.i32_type(),
                                    llvm_str!(""),
                                )
                            };
                            Ok(cmp_i32)
                        }
                        _ => {
                            return Err(format!("Misidentified binary expression"));
                        }
                    }
                }
            }

            Expression::UnaryExpression { op, expression } => {
                trace!("Generating unary expression");
                match &op[..] {
                    "-" => unsafe {
                        Ok(core::LLVMBuildNeg(
                            self.builder,
                            self.gen_expression(expression)?,
                            llvm_str!(""),
                        ))
                    },
                    _ => {
                        return Err(format!("Misidentified unary expression"));
                    }
                }
            }
        }
    }
}
