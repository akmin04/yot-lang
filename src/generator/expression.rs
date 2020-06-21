use crate::c_str;
use crate::generator::Generator;
use crate::lexer::tokens::Literal;
use crate::parser::expression::Expression;
use crate::Result;
use llvm_sys::core;
use llvm_sys::prelude::LLVMValueRef;
use llvm_sys::LLVMIntPredicate;
use log::trace;

impl Generator {
    pub unsafe fn gen_expression(&self, expression: &Expression) -> Result<LLVMValueRef> {
        trace!("Generating expression");
        match expression {
            Expression::LiteralExpression { value } => {
                trace!("Generating literal expression: {:?}", value);
                match value {
                    Literal::Integer(i) => {
                        trace!("Integer literal: {}", i);
                        Ok(core::LLVMConstInt(self.i32_type(), *i as u64, false as i32))
                    }
                    Literal::Str(s) => {
                        trace!("Str literal: {}", s);
                        Ok(core::LLVMConstString(
                            c_str!(s),
                            s.len() as u32,
                            false as i32,
                        ))
                    }
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
                    Ok(core::LLVMBuildLoad2(
                        self.builder,
                        self.i32_type(),
                        *var,
                        c_str!(""),
                    ))
                } else {
                    Err(format!("Unresolved variable reference `{}`", name))
                }
            }

            Expression::FunctionCallExpression { name, args } => {
                trace!("Generating function call expression: {}", name);
                let mut llvm_args: Vec<LLVMValueRef> = Vec::new();
                for arg in args {
                    llvm_args.push(self.gen_expression(arg)?);
                }

                let function = core::LLVMGetNamedFunction(self.module, c_str!(name));
                if function.is_null() {
                    return Err(format!("Function `{}` doesn't exist", name));
                }
                Ok(core::LLVMBuildCall(
                    self.builder,
                    function,
                    llvm_args.as_mut_ptr(),
                    args.len() as u32,
                    c_str!(""),
                ))
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

                        core::LLVMBuildStore(self.builder, r, *var);

                        Ok(r)
                    } else {
                        Err("Expected variable reference on assignment".to_string())
                    }
                } else {
                    let l = self.gen_expression(l_expression)?;

                    match &op[..] {
                        "+" => Ok(core::LLVMBuildAdd(self.builder, l, r, c_str!(""))),
                        "-" => Ok(core::LLVMBuildSub(self.builder, l, r, c_str!(""))),
                        "*" => Ok(core::LLVMBuildMul(self.builder, l, r, c_str!(""))),
                        "/" => Ok(core::LLVMBuildSDiv(self.builder, l, r, c_str!(""))),
                        "==" | "!=" | "<" | ">" | "<=" | ">=" => {
                            let cmp = {
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
                                    c_str!(""),
                                )
                            };
                            // Cast i1 to i32
                            let cmp_i32 = {
                                core::LLVMBuildZExt(self.builder, cmp, self.i32_type(), c_str!(""))
                            };
                            Ok(cmp_i32)
                        }
                        _ => Err("Misidentified binary expression".to_string()),
                    }
                }
            }

            Expression::UnaryExpression { op, expression } => {
                trace!("Generating unary expression");
                match &op[..] {
                    "-" => Ok(core::LLVMBuildNeg(
                        self.builder,
                        self.gen_expression(expression)?,
                        c_str!(""),
                    )),
                    _ => Err("Misidentified unary expression".to_string()),
                }
            }
        }
    }
}
