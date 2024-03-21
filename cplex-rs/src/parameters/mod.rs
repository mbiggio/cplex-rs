use std::ffi::{c_double, c_int};

use ffi::{CPX_PARAM_EPGAP, CPX_PARAM_PARALLELMODE, CPX_PARAM_SCRIND, CPX_PARAM_THREADS};

/// Parameter trait
pub trait Parameter {
    fn value(&self) -> ParameterValue;
    fn id(&self) -> u32;
}

#[derive(Copy, Clone, Debug)]
pub enum ParameterValue {
    Integer(c_int),
    Double(c_double),
}

/// Parallel Mode
#[derive(Copy, Clone, Debug)]
pub enum ParallelMode {
    Opportunistic,
    Auto,
    Deterministic,
}

impl Parameter for ParallelMode {
    fn value(&self) -> ParameterValue {
        ParameterValue::Integer(match self {
            ParallelMode::Opportunistic => -1,
            ParallelMode::Auto => 0,
            ParallelMode::Deterministic => 1,
        })
    }

    fn id(&self) -> u32 {
        CPX_PARAM_PARALLELMODE
    }
}

/// Threads
#[derive(Copy, Clone, Debug)]
pub struct Threads(pub i32);

impl Parameter for Threads {
    fn value(&self) -> ParameterValue {
        ParameterValue::Integer(self.0)
    }

    fn id(&self) -> u32 {
        CPX_PARAM_THREADS
    }
}

/// ScreenOutput
#[derive(Copy, Clone, Debug)]
pub struct ScreenOutput(pub bool);

impl Parameter for ScreenOutput {
    fn value(&self) -> ParameterValue {
        ParameterValue::Integer(if self.0 { 1 } else { 0 })
    }

    fn id(&self) -> u32 {
        CPX_PARAM_SCRIND
    }
}

/// RelativeGap
#[derive(Copy, Clone, Debug)]
pub struct RelativeGap(pub f64);

impl Parameter for RelativeGap {
    fn value(&self) -> ParameterValue {
        ParameterValue::Double(self.0)
    }

    fn id(&self) -> u32 {
        CPX_PARAM_EPGAP
    }
}
