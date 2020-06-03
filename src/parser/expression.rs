use crate::lexer::tokens;
use crate::lexer::tokens::{Literal, Token, UNARY_SYMBOLS};
use crate::parser::Parser;
use crate::Result;
use crate::{peek_identifier_or_err, peek_literal_or_err, peek_symbol_or_err};
use log::trace;

/// A yot expression.
#[derive(Debug)]
pub enum Expression {
    /// A literal value.
    ///
    /// # Grammar
    /// * IntegerLiteral
    /// * StrLiteral
    LiteralExpression { value: Literal },

    /// An expression enclosed in parentheses.
    ///
    /// # Grammar
    /// * "(" + Expression + ")"
    ParenExpression { expression: Box<Expression> },

    /// A reference to a variable.
    ///
    /// # Grammar
    /// * Identifier
    VariableReferenceExpression { name: String },

    /// A call to a function with arguments.
    ///
    /// # Grammar
    /// * Identifier + "(" + (Expression + ",")... + ")"
    FunctionCallExpression { name: String, args: Vec<Expression> },

    /// A link between two expresesions with a binary operator.
    ///
    /// Possible operators:
    /// "=", "+", "-", "*", "/", "==", "!=", "<", ">", "<=", ">="
    ///
    /// # Grammar
    /// * Expression + op + Expression
    BinaryExpression {
        op: String,
        l_expression: Box<Expression>,
        r_expression: Box<Expression>,
    },

    /// A prefix operator to an expression.
    ///
    /// Possible operators:
    /// "-"
    ///
    /// # Grammar
    /// * op + Expression
    UnaryExpression {
        op: String,
        expression: Box<Expression>,
    },
}

impl Parser {
    pub fn parse_expression(&mut self) -> Result<Expression> {
        trace!("Parsing expression");
        let l_expression = self.parse_expression_no_binary()?;
        self.parse_binary_r_expression(0, l_expression)
    }

    fn parse_expression_no_binary(&mut self) -> Result<Expression> {
        match self.tokens.peek() {
            Some(Token::Literal(_)) => self.parse_literal_expression(),
            Some(Token::Identifier(_)) => {
                let name = peek_identifier_or_err!(self);
                self.tokens.next();
                if self.next_symbol_is("(") {
                    self.parse_function_call_expression(name)
                } else {
                    self.parse_variable_reference_expression(name)
                }
            }
            Some(Token::Symbol(s)) if s == "(" => self.parse_paren_expression(),
            Some(Token::Symbol(s)) if UNARY_SYMBOLS.contains(&&s[..]) => {
                self.parse_unary_expression()
            }
            _ => Err("Unable to parse expression".to_string()),
        }
    }

    fn parse_literal_expression(&mut self) -> Result<Expression> {
        trace!("Parsing literal expression");
        let expression = Ok(Expression::LiteralExpression {
            value: peek_literal_or_err!(self),
        });
        self.tokens.next();
        expression
    }

    fn parse_paren_expression(&mut self) -> Result<Expression> {
        trace!("Parsing paren expression");
        if !self.next_symbol_is("(") {
            return Err("Misidentified paren expression".to_string());
        }
        let expression = Box::new(self.parse_expression()?);
        if !self.next_symbol_is(")") {
            return Err("Expected `)` after expression".to_string());
        }
        Ok(Expression::ParenExpression { expression })
    }

    fn parse_variable_reference_expression(&mut self, name: String) -> Result<Expression> {
        trace!("Parsing variable reference expression");
        Ok(Expression::VariableReferenceExpression { name })
    }

    fn parse_function_call_expression(&mut self, name: String) -> Result<Expression> {
        trace!("Parsing function call expression");
        let mut args: Vec<Expression> = Vec::new();

        if !self.next_symbol_is(")") {
            loop {
                args.push(self.parse_expression()?);
                match self.tokens.next() {
                    Some(Token::Symbol(s)) if s == ")" => break,
                    Some(Token::Symbol(s)) if s == "," => (),
                    _ => {
                        return Err(format!(
                            "Expected `)` or `,` after function call `{}`",
                            name
                        ))
                    }
                }
            }
        }
        Ok(Expression::FunctionCallExpression { name, args })
    }

    fn parse_binary_r_expression(
        &mut self,
        precedence: i32,
        l_expression: Expression,
    ) -> Result<Expression> {
        trace!("Parsing binary r expression");
        let mut l_expression = l_expression;

        macro_rules! peek_symbol_or_zero {
            ($self:ident) => {
                String::from(match $self.tokens.peek() {
                    Some(Token::Symbol(s)) => s,
                    _ => "0",
                });
            };
        }

        loop {
            let op = peek_symbol_or_zero!(self);
            let current_precedence = tokens::binary_op_precedence(&op);

            if current_precedence < precedence {
                return Ok(l_expression);
            }

            self.tokens.next();
            let mut r_expression = self.parse_expression_no_binary()?;

            let next_precedence = tokens::binary_op_precedence(&peek_symbol_or_zero!(self));

            if current_precedence < next_precedence {
                r_expression =
                    self.parse_binary_r_expression(current_precedence + 1, r_expression)?;
            }

            l_expression = Expression::BinaryExpression {
                op,
                l_expression: Box::new(l_expression),
                r_expression: Box::new(r_expression),
            };
        }
    }

    fn parse_unary_expression(&mut self) -> Result<Expression> {
        trace!("Parsing unary expression");
        let op = peek_symbol_or_err!(self);
        self.tokens.next();
        let expression = Box::new(self.parse_expression_no_binary()?);
        Ok(Expression::UnaryExpression { op, expression })
    }
}
