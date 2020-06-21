use crate::lexer::tokens::Token;
use crate::parser::expression::Expression;
use crate::parser::Parser;
use crate::peek_identifier_or_err;
use crate::Result;
use log::{debug, trace};

/// A yot statement.
#[derive(Debug)]
pub enum Statement {
    /// Multiple statements enclosed in braces.
    ///
    /// # Grammar
    /// * "{" + Statement... + "}"
    CompoundStatement { statements: Vec<Statement> },

    /// An if/else statement.
    ///
    /// # Grammar
    /// * "?" + "[" + Expression + "]" + Statement
    /// * "?" + "[" + Expression + "]" + Statement + ":" + Statement
    IfStatement {
        condition: Box<Expression>,
        then_statement: Box<Statement>,
        else_statement: Option<Box<Statement>>,
    },

    /// A return statement.
    ///
    /// # Grammar
    /// * "->" + Expression + ";"
    ReturnStatement { value: Box<Expression> },

    /// A variable declaration with an optional value.
    ///
    /// # Grammar
    /// * "@" + Identifier + ";"
    /// * "@" + Identifier + "=" + Expression + ";"
    VariableDeclarationStatement {
        name: String,
        value: Option<Box<Expression>>,
    },

    /// An expression ending with a semicolon.
    ///
    /// # Grammar
    /// * Expression + ";"
    ExpressionStatement { expression: Box<Expression> },

    /// A no-op ending (just a semicolon).
    ///
    /// # Grammar
    /// * ;
    NoOpStatement,
}

impl Parser {
    pub fn parse_statement(&mut self) -> Result<Statement> {
        trace!("Parsing statement");
        match self.tokens.peek() {
            Some(Token::Symbol(s)) if s == "{" => self.parse_compound_statement(),
            Some(Token::Symbol(s)) if s == "?" => self.parse_if_statement(),
            Some(Token::Symbol(s)) if s == "->" => self.parse_return_statement(),
            Some(Token::Symbol(s)) if s == "@" => self.parse_variable_declaration_statement(),
            Some(Token::Symbol(s)) if s == ";" => self.parse_no_op_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_compound_statement(&mut self) -> Result<Statement> {
        trace!("Parsing compound statement");
        self.tokens.next(); // Eat {
        let mut statements: Vec<Statement> = Vec::new();
        while !self.next_symbol_is("}") {
            statements.push(self.parse_statement()?);
        }
        Ok(Statement::CompoundStatement { statements })
    }

    fn parse_if_statement(&mut self) -> Result<Statement> {
        trace!("Parsing if statement");
        self.tokens.next(); // Eat ?
        if !self.next_symbol_is("[") {
            return Err("Expected `[` after `?` in if statement".to_string());
        }

        let condition = Box::new(self.parse_expression()?);
        if !self.next_symbol_is("]") {
            return Err("Expected `]` after condition in if statement".to_string());
        }
        let then_statement = Box::new(self.parse_statement()?);
        let else_statement = if self.next_symbol_is(":") {
            debug!("Else statement found");
            Some(Box::new(self.parse_statement()?))
        } else {
            debug!("No else statement found");
            None
        };

        Ok(Statement::IfStatement {
            condition,
            then_statement,
            else_statement,
        })
    }

    fn parse_return_statement(&mut self) -> Result<Statement> {
        trace!("Parsing return statement");
        self.tokens.next(); // Eat ->
        let value = Box::new(self.parse_expression()?);

        if !self.next_symbol_is(";") {
            return Err("Expected `;` after return statement".to_string());
        }

        Ok(Statement::ReturnStatement { value })
    }

    fn parse_variable_declaration_statement(&mut self) -> Result<Statement> {
        trace!("Parsing variable declaration statement");
        self.tokens.next(); // Eat @
        let name = peek_identifier_or_err!(self);
        self.tokens.next();

        let value = if self.next_symbol_is("=") {
            trace!("Found expression after");
            Some(Box::new(self.parse_expression()?))
        } else {
            trace!("No expression found after");
            None
        };

        if !self.next_symbol_is(";") {
            return Err("Expected `;` after variable declaration statement".to_string());
        }
        Ok(Statement::VariableDeclarationStatement { name, value })
    }

    fn parse_expression_statement(&mut self) -> Result<Statement> {
        trace!("Parsing expression statement");
        let expression = Box::new(self.parse_expression()?);
        if !self.next_symbol_is(";") {
            return Err("Expected `;` after expression statement".to_string());
        }
        Ok(Statement::ExpressionStatement { expression })
    }

    fn parse_no_op_statement(&mut self) -> Result<Statement> {
        trace!("Parsing no op statement");
        self.tokens.next();
        Ok(Statement::NoOpStatement)
    }
}
