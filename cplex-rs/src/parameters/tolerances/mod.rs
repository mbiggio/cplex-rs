use ffi::{CPXPARAM_MIP_Tolerances_AbsMIPGap, CPXPARAM_MIP_Tolerances_MIPGap};

use crate::errors::{self, Result};
use crate::parameters::{private, Parameter, ParameterValue};

impl private::Parameter for MIPGap {}
impl private::Parameter for AbsMIPGap {}

/// Relative MIP gap tolerance
/// <https://www.ibm.com/docs/en/icos/12.9.0?topic=parameters-relative-mip-gap-tolerance>
#[derive(Copy, Clone, Debug)]
pub struct MIPGap(f64);

impl MIPGap {
    pub fn new(value: f64) -> Result<Self> {
        if !(0.0..=1.0).contains(&value) {
            return Err(errors::Input::from_message(
                "CPXPARAM_MIP_Tolerances_MIPGap cannot be < 0.0 or > 1.0".to_string(),
            )
            .into());
        }
        Ok(Self(value))
    }
}

impl Parameter for MIPGap {
    fn value(&self) -> ParameterValue {
        ParameterValue::Double(self.0)
    }

    fn id(&self) -> u32 {
        CPXPARAM_MIP_Tolerances_MIPGap
    }
}

/// Absolute MIP gap tolerance
/// <https://www.ibm.com/docs/en/icos/12.9.0?topic=parameters-absolute-mip-gap-tolerance>
#[derive(Copy, Clone, Debug)]
pub struct AbsMIPGap(f64);

impl AbsMIPGap {
    pub fn new(value: f64) -> Result<Self> {
        if value < 0.0 {
            return Err(errors::Input::from_message(
                "CPXPARAM_MIP_Tolerances_AbsMIPGap cannot be < 0.0".to_string(),
            )
            .into());
        }
        Ok(Self(value))
    }
}

impl Parameter for AbsMIPGap {
    fn value(&self) -> ParameterValue {
        ParameterValue::Double(self.0)
    }

    fn id(&self) -> u32 {
        CPXPARAM_MIP_Tolerances_AbsMIPGap
    }
}
