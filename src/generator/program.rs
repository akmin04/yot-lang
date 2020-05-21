use crate::generator::Generator;
use crate::parser::program::Program;
use log::info;

impl Generator {
    pub fn gen_program(&self, program: &Program) -> Result<(), &'static str> {
        info!("Generating program");
        for function in &program.functions {
            // Clear variables and arguments
            let mut local_vars_mut = self.local_vars.borrow_mut();
            let mut fn_args_mut = self.fn_args.borrow_mut();
            local_vars_mut.clear();
            fn_args_mut.clear();
            drop(local_vars_mut);
            drop(fn_args_mut);

            self.gen_function(&function)?;
        }
        Ok(())
    }
}
