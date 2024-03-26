use ffi::{
    CPXPARAM_Barrier_Limits_Corrections, CPXPARAM_Barrier_Limits_Growth,
    CPXPARAM_Barrier_Limits_Iteration, CPXPARAM_Barrier_Limits_ObjRange,
};

use crate::errors::{self, Result};
use crate::parameters::{private, Parameter, ParameterValue};

impl private::Parameter for Growth {}
impl private::Parameter for Iteration {}
impl private::Parameter for Corrections {}
impl private::Parameter for ObjRange {}
impl private::Parameter for Ordering {}

/// Barrier growth limit.
/// <https://www.ibm.com/docs/en/icos/22.1.1?topic=parameters-barrier-growth-limit>
#[derive(Copy, Clone, Debug)]
pub struct Growth(f64);

impl Growth {
    pub fn new(value: f64) -> Result<Self> {
        if value < 1.0 {
            return Err(errors::Input::from_message(
                "CPXPARAM_Barrier_Limits_Growth cannot be < 1.0".to_string(),
            )
            .into());
        }
        Ok(Self(value))
    }
}

impl Parameter for Growth {
    fn value(&self) -> ParameterValue {
        ParameterValue::Double(self.0)
    }

    fn id(&self) -> u32 {
        CPXPARAM_Barrier_Limits_Growth
    }
}

/// Barrier iteration limit.
/// <https://www.ibm.com/docs/en/icos/22.1.1?topic=parameters-barrier-iteration-limit>
#[derive(Copy, Clone, Debug)]
pub struct Iteration(pub u64);

impl Parameter for Iteration {
    fn value(&self) -> ParameterValue {
        ParameterValue::Long(self.0 as i64)
    }

    fn id(&self) -> u32 {
        CPXPARAM_Barrier_Limits_Iteration
    }
}

/// Barrier maximum correction limit.
/// <https://www.ibm.com/docs/en/icos/22.1.1?topic=parameters-barrier-maximum-correction-limit>
#[derive(Copy, Clone, Debug)]
pub enum Corrections {
    Automatic,
    Number(u64),
}

impl Parameter for Corrections {
    fn value(&self) -> ParameterValue {
        ParameterValue::Long(match self {
            Self::Automatic => -1,
            &Self::Number(n) => n as i64,
        })
    }

    fn id(&self) -> u32 {
        CPXPARAM_Barrier_Limits_Corrections
    }
}

/// Barrier objective range.
/// <https://www.ibm.com/docs/en/icos/22.1.1?topic=parameters-barrier-maximum-correction-limit>
#[derive(Copy, Clone, Debug)]
pub struct ObjRange(f64);

impl ObjRange {
    pub fn new(value: f64) -> Result<Self> {
        if value < 0.0 {
            return Err(errors::Input::from_message(
                "CPXPARAM_Barrier_Limits_ObjRange cannot be < 0.0".to_string(),
            )
            .into());
        }
        Ok(Self(value))
    }
}

impl Parameter for ObjRange {
    fn value(&self) -> ParameterValue {
        ParameterValue::Double(self.0)
    }

    fn id(&self) -> u32 {
        CPXPARAM_Barrier_Limits_ObjRange
    }
}

/// Barrier ordering algorithm.
/// <https://www.ibm.com/docs/en/icos/22.1.1?topic=parameters-barrier-ordering-algorithm>
#[derive(Copy, Clone, Debug)]
pub enum Ordering {
    Automatic,
    ApproximateMinimumDegree,
    ApproximateMinimumFill,
    NestedDissection,
}

impl Parameter for Ordering {
    fn value(&self) -> ParameterValue {
        ParameterValue::Integer(match self {
            Self::Automatic => 0,
            Self::ApproximateMinimumDegree => 1,
            Self::ApproximateMinimumFill => 2,
            Self::NestedDissection => 3,
        })
    }

    fn id(&self) -> u32 {
        CPXPARAM_Barrier_Limits_Corrections
    }
}
