use std::{
    ffi::{c_int, CString},
    ops::Not,
};

pub type Result<T> = std::result::Result<T, Error>;

use ffi::{cpxenv, CPXgeterrorstring, CPXMESSAGEBUFSIZE};
use thiserror::Error;

#[macro_export]
macro_rules! cpx_result {
    ( unsafe { $func:ident ( $env:expr $(, $b:expr)* $(,)? ) } ) => {
        {
            let status = unsafe { $func($env $(,$b)* ) };
            if status != 0 {
                Err(crate::errors::Error::from(crate::errors::Cplex::from_code($env, status)))
            } else {
                Ok(())
            }
        }
    };
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Cplex error: {0}")]
    Cplex(#[from] Cplex),
    #[error("Input error: {0}")]
    Input(#[from] Input),
}

#[derive(Error, Debug)]
#[error("Cplex error status {code}: {message}")]
pub struct Cplex {
    pub code: i32,
    pub message: String,
}

impl Cplex {
    pub(crate) fn from_code(env: *const cpxenv, code: c_int) -> Cplex {
        let mut buf = vec![0u8; CPXMESSAGEBUFSIZE as usize];
        let ptr = unsafe { CPXgeterrorstring(env, code, buf.as_mut_ptr() as *mut i8) };
        let message = ptr
            .is_null()
            .not()
            .then_some(())
            .and_then(|_| CString::from_vec_with_nul(buf).ok())
            .and_then(|cs| cs.into_string().ok())
            .unwrap_or_else(|| "Unable to extract error message".to_string());
        Self { code, message }
    }

    pub(crate) fn env_error(code: c_int) -> Cplex {
        let message = "Error encountered when constructing CPLEX env".to_owned();
        Self { code, message }
    }
}

#[derive(Error, Debug)]
#[error("Input error: {message}")]
pub struct Input {
    pub message: String,
}

impl Input {
    pub(crate) fn from_message(message: String) -> Input {
        Self { message }
    }
}
