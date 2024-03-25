use std::{
    ffi::{c_int, CString},
    ops::Not,
};

pub type Result<T> = std::result::Result<T, Error>;

use ffi::{cpxenv, cpxlp, CPXgeterrorstring, CPXgetijdiv, CPXsolninfo, CPXMESSAGEBUFSIZE};
use log::error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Cplex error: {0}")]
    Cplex(#[from] Cplex),
    #[error("Input error: {0}")]
    Input(#[from] Input),
}

#[derive(Error, Debug)]
#[error("Cplex error status {code}: {message}")]
pub enum Cplex {
    Unbounded { code: c_int, message: String },
    Unfeasible { code: c_int, message: String },
    Other { code: c_int, message: String },
}

impl Cplex {
    pub(crate) fn from_code(env: *const cpxenv, lp: *const cpxlp, code: c_int) -> Cplex {
        let mut buf = vec![0u8; CPXMESSAGEBUFSIZE as usize];
        let ptr = unsafe { CPXgeterrorstring(env, code, buf.as_mut_ptr() as *mut i8) };
        let message = ptr
            .is_null()
            .not()
            .then_some(())
            .and_then(|_| CString::from_vec_with_nul(buf).ok())
            .and_then(|cs| cs.into_string().ok())
            .unwrap_or_else(|| "Unable to extract error message".to_string());

        if lp.is_null() {
            return Self::Other { code, message };
        }

        if !Self::is_feasible(env, lp) {
            Self::Unfeasible { code, message }
        } else if !Self::is_bounded(env, lp) {
            Self::Unbounded { code, message }
        } else {
            Self::Other { code, message }
        }
    }

    fn is_bounded(env: *const cpxenv, lp: *const cpxlp) -> bool {
        let mut i = 0;
        let mut j = 0;
        match unsafe { CPXgetijdiv(env, lp, &mut i, &mut j) } {
            0 => j == -1,
            _ => {
                error!("Unable to determine if problem is bounded, assuming it is");
                true
            }
        }
    }

    fn is_feasible(env: *const cpxenv, lp: *const cpxlp) -> bool {
        let mut lpstat = 0;
        let mut stype = 0;
        let mut pfeas = 0;
        let mut dfeas = 0;
        match unsafe { CPXsolninfo(env, lp, &mut lpstat, &mut stype, &mut pfeas, &mut dfeas) } {
            0 => pfeas != 0,
            _ => {
                error!("Unable to determine if problem is feasible, assuming it is");
                true
            }
        }
    }

    pub(crate) fn env_error(code: c_int) -> Cplex {
        let message = "Error encountered when constructing CPLEX env".to_owned();
        Self::Other { code, message }
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
