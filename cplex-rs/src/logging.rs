use std::ffi::{c_char, c_void, CStr};

pub type LoggingCallback = Option<unsafe extern "C" fn(*mut c_void, *const c_char)>;
pub type LoggingClosure = Box<dyn Fn(&str) + Send>;

pub(crate) const RESULTS_STREAM_IDX: usize = 0;
pub(crate) const WARNING_STREAM_IDX: usize = 1;
pub(crate) const ERROR_STREAM_IDX: usize = 2;
pub(crate) const LOG_STREAM_IDX: usize = 3;

pub(crate) const DEFAULT_LOGGING_CLOSURE: Option<(LoggingClosure, LoggingCallback)> = None;

#[derive(Clone, Copy, Debug)]
pub enum StreamType {
    Results,
    Warning,
    Error,
    Log,
}

impl StreamType {
    pub(crate) fn as_index(&self) -> usize {
        match self {
            Self::Results => RESULTS_STREAM_IDX,
            Self::Warning => WARNING_STREAM_IDX,
            Self::Error => ERROR_STREAM_IDX,
            Self::Log => LOG_STREAM_IDX,
        }
    }
}

pub(crate) fn get_trampoline<F: Fn(&str)>() -> LoggingCallback {
    Some(trampoline::<F>)
}

unsafe extern "C" fn trampoline<F>(user_data: *mut c_void, msg: *const c_char)
where
    F: Fn(&str),
{
    let logging_closure = &mut *(user_data as *mut F);
    let msg = CStr::from_ptr(msg);
    for line in msg.to_string_lossy().lines() {
        logging_closure(line);
    }
}
