use std::ffi::{c_void, CString};

use crate::{
    errors::{self, Result},
    logging::{
        get_trampoline, LoggingCallback, LoggingClosure, StreamType, DEFAULT_LOGGING_CLOSURE,
    },
    parameters::{Parameter, ParameterValue},
};
use ffi::{
    cpxchannel, cpxenv, CPXaddfuncdest, CPXcloseCPLEX, CPXdelfuncdest, CPXgetchannels,
    CPXopenCPLEX, CPXsetdblparam, CPXsetintparam, CPXsetlongparam, CPXsetstrparam,
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

pub struct Environment {
    pub(crate) inner: *mut cpxenv,
    pub(crate) logging_closures: [Option<(LoggingClosure, LoggingCallback)>; 4],
}

unsafe impl Send for Environment {}

impl Environment {
    pub fn new() -> Result<Environment> {
        let mut status = 0;
        let inner = unsafe { CPXopenCPLEX(&mut status) };
        if inner.is_null() {
            Err(errors::Cplex::env_error(status).into())
        } else {
            let env = Environment {
                inner,
                logging_closures: [DEFAULT_LOGGING_CLOSURE; 4],
            };

            Ok(env)
        }
    }

    pub fn set_parameter<P: Parameter>(&mut self, p: P) -> Result<()> {
        match p.value() {
            ParameterValue::Integer(i) => {
                macros::cpx_env_result!(unsafe { CPXsetintparam(self.inner, p.id() as i32, i) })
            }
            ParameterValue::Long(l) => {
                macros::cpx_env_result!(unsafe { CPXsetlongparam(self.inner, p.id() as i32, l) })
            }
            ParameterValue::Double(d) => {
                macros::cpx_env_result!(unsafe { CPXsetdblparam(self.inner, p.id() as i32, d) })
            }
            ParameterValue::String(s) => {
                let cstr = CString::new(s.as_bytes()).expect("Invalid parameter string");
                macros::cpx_env_result!(unsafe {
                    CPXsetstrparam(self.inner, p.id() as i32, cstr.as_ptr())
                })
            }
        }
    }

    pub fn unset_logging_closure(&mut self, stream_type: StreamType) -> Result<()> {
        let channel = self.channel_from_stream_type(stream_type)?;

        assert!(!channel.is_null());

        if let Some((mut previous_closure, previous_trampoline)) =
            self.logging_closures[stream_type.as_index()].take()
        {
            macros::cpx_env_result!(unsafe {
                CPXdelfuncdest(
                    self.inner,
                    channel,
                    &mut *previous_closure as *mut _ as *mut c_void,
                    previous_trampoline,
                )
            })?;
        }

        Ok(())
    }

    pub fn set_logging_closure<F: Fn(&str) + Send + 'static>(
        &mut self,
        stream_type: StreamType,
        closure: F,
    ) -> Result<()> {
        let channel = self.channel_from_stream_type(stream_type)?;

        assert!(!channel.is_null());

        if let Some((mut previous_closure, previous_trampoline)) =
            self.logging_closures[stream_type.as_index()].take()
        {
            macros::cpx_env_result!(unsafe {
                CPXdelfuncdest(
                    self.inner,
                    channel,
                    &mut *previous_closure as *mut _ as *mut c_void,
                    previous_trampoline,
                )
            })?;
        }

        let mut new_closure = Box::new(closure);
        let new_trampoline = get_trampoline::<F>();
        macros::cpx_env_result!(unsafe {
            CPXaddfuncdest(
                self.inner,
                channel,
                &mut *new_closure as *mut F as *mut c_void,
                new_trampoline,
            )
        })?;

        self.logging_closures[stream_type.as_index()] = Some((new_closure, new_trampoline));

        Ok(())
    }

    fn channel_from_stream_type(&self, stream_type: StreamType) -> Result<*mut cpxchannel> {
        let mut results_channel = std::ptr::null_mut();
        let mut warning_channel = std::ptr::null_mut();
        let mut error_channel = std::ptr::null_mut();
        let mut log_channel = std::ptr::null_mut();
        macros::cpx_env_result!(unsafe {
            CPXgetchannels(
                self.inner,
                &mut results_channel,
                &mut warning_channel,
                &mut error_channel,
                &mut log_channel,
            )
        })?;

        Ok(match stream_type {
            StreamType::Error => error_channel,
            StreamType::Log => log_channel,
            StreamType::Results => results_channel,
            StreamType::Warning => warning_channel,
        })
    }
}

impl Drop for Environment {
    fn drop(&mut self) {
        self.unset_logging_closure(StreamType::Log).unwrap();
        self.unset_logging_closure(StreamType::Results).unwrap();
        self.unset_logging_closure(StreamType::Warning).unwrap();
        self.unset_logging_closure(StreamType::Error).unwrap();
        unsafe {
            let status = CPXcloseCPLEX(&mut self.inner);
            if status != 0 {
                error!("Unable to close CPLEX context, got status: '{}'", status)
            }
        }
    }
}
