use crate::{
    cpx_result,
    errors::{self, Result},
    parameters::{Parameter, ParameterValue},
};
use ffi::{cpxenv, CPXcloseCPLEX, CPXopenCPLEX, CPXsetdblparam, CPXsetintparam};
use log::error;

pub struct Environment(pub(crate) *mut cpxenv);

impl Environment {
    pub fn new() -> Result<Environment> {
        let mut status = 0;
        let env = unsafe { CPXopenCPLEX(&mut status) };
        if env.is_null() {
            Err(errors::Cplex::env_error(status).into())
        } else {
            Ok(Environment(env))
        }
    }

    pub fn set_parameter<P: Parameter>(&mut self, p: P) -> Result<()> {
        match p.value() {
            ParameterValue::Integer(i) => {
                cpx_result!(unsafe { CPXsetintparam(self.0, p.id() as i32, i) })
            }
            ParameterValue::Double(d) => {
                cpx_result!(unsafe { CPXsetdblparam(self.0, p.id() as i32, d) })
            }
        }
    }
}

impl Drop for Environment {
    fn drop(&mut self) {
        unsafe {
            let status = CPXcloseCPLEX(&mut self.0);
            if status != 0 {
                error!("Unable to close CPLEX context, got status: '{}'", status)
            }
        }
    }
}
