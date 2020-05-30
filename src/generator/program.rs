use crate::generator::Generator;
use crate::parser::program::Program;
use crate::Result;
use log::info;

impl Generator {
    pub fn gen_program(&self, program: &Program) -> Result<()> {
        info!("Generating program");
        for function in &program.functions {
            self.local_vars.borrow_mut().clear();
            self.gen_function(&function)?;
        }
        Ok(())
    }
}
