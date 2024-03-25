use std::ffi::CString;

use crate::{
    errors::{self, Result},
    parameters::{Parameter, ParameterValue},
};
use ffi::{
    cpxenv, CPXcloseCPLEX, CPXopenCPLEX, CPXsetdblparam, CPXsetintparam, CPXsetlongparam,
    CPXsetstrparam,
};
use log::error;

mod macros {
    macro_rules! cpx_env_result {
        ( unsafe { $func:ident ( $env:expr $(, $b:expr)* $(,)? ) } ) => {
            {
                let status = unsafe { $func( $env $(,$b)* ) };
                if status != 0 {
                    Err(errors::Error::from(errors::Cplex::from_code($env, std::ptr::null(), status)))
                } else {
                    Ok(())
                }
            }
        };
    }

    pub(super) use cpx_env_result;
}

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
                macros::cpx_env_result!(unsafe { CPXsetintparam(self.0, p.id() as i32, i) })
            }
            ParameterValue::Long(l) => {
                macros::cpx_env_result!(unsafe { CPXsetlongparam(self.0, p.id() as i32, l) })
            }
            ParameterValue::Double(d) => {
                macros::cpx_env_result!(unsafe { CPXsetdblparam(self.0, p.id() as i32, d) })
            }
            ParameterValue::String(s) => {
                let cstr = CString::new(s.as_bytes()).expect("Invalid parameter string");
                macros::cpx_env_result!(unsafe {
                    CPXsetstrparam(self.0, p.id() as i32, cstr.as_ptr())
                })
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
