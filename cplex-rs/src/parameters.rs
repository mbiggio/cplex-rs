use std::ffi::{c_double, c_int};

use ffi::{CPXPARAM_Threads, CPX_PARAM_EPGAP, CPX_PARAM_PARALLELMODE, CPX_PARAM_SCRIND};

#[derive(Copy, Clone, Debug)]
pub enum ParallelMode {
    Opportunistic,
    Auto,
    Deterministic,
}

#[derive(Copy, Clone, Debug)]
pub enum Parameter {
    Threads(u64),
    ScreenOutput(bool),
    RelativeGap(f64),
    ParallelMode(ParallelMode),
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum ParameterType {
    Integer(c_int),
    Double(c_double),
}

impl Parameter {
    pub(crate) fn to_id(self) -> c_int {
        let id = match self {
            Self::Threads(_) => CPXPARAM_Threads,
            Self::ScreenOutput(_) => CPX_PARAM_SCRIND,
            Self::RelativeGap(_) => CPX_PARAM_EPGAP,
            Self::ParallelMode(_) => CPX_PARAM_PARALLELMODE,
        };

        id as c_int
    }

    pub(crate) fn param_type(&self) -> ParameterType {
        match *self {
            Self::Threads(t) => ParameterType::Integer(t as c_int),
            Self::ScreenOutput(b) => ParameterType::Integer(if b { 1 } else { 0 }),
            Self::RelativeGap(g) => ParameterType::Double(g as c_double),
            Self::ParallelMode(p) => ParameterType::Integer(match p {
                ParallelMode::Opportunistic => -1,
                ParallelMode::Auto => 0,
                ParallelMode::Deterministic => 1,
            }),
        }
    }
}
