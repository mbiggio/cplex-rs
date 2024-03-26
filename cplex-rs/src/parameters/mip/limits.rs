use ffi::{
    CPXPARAM_MIP_Limits_AggForCut, CPXPARAM_MIP_Limits_Nodes, CPXPARAM_MIP_Limits_Solutions,
};

use crate::{
    errors::{self, Result},
    parameters::{private, Parameter, ParameterValue},
};

impl private::Parameter for AggForCut {}
impl private::Parameter for Solutions {}
impl private::Parameter for Nodes {}

/// AggForCut.
/// <https://www.ibm.com/docs/en/icos/22.1.1?topic=parameters-constraint-aggregation-limit-cut-generation>
#[derive(Copy, Clone, Debug)]
pub struct AggForCut(pub u32);

impl Parameter for AggForCut {
    fn value(&self) -> ParameterValue {
        ParameterValue::Integer(self.0 as i32)
    }

    fn id(&self) -> u32 {
        CPXPARAM_MIP_Limits_AggForCut
    }
}

/// Solutions.
/// <https://www.ibm.com/docs/en/icos/22.1.1?topic=parameters-mip-integer-solution-limit>
#[derive(Copy, Clone, Debug)]
pub struct Solutions(u64);

impl Solutions {
    pub fn new(value: u64) -> Result<Self> {
        if value == 0 {
            return Err(errors::Input::from_message(
                "CPXPARAM_MIP_Limits_Solutions cannot be == 0".to_string(),
            )
            .into());
        }

        Ok(Self(value))
    }
}

impl Parameter for Solutions {
    fn value(&self) -> ParameterValue {
        ParameterValue::Long(self.0 as i64)
    }

    fn id(&self) -> u32 {
        CPXPARAM_MIP_Limits_Solutions
    }
}

/// MIP node limit.
/// <https://www.ibm.com/docs/en/icos/12.9.0?topic=parameters-mip-node-limit>
#[derive(Copy, Clone, Debug)]
pub struct Nodes(pub u64);

impl Parameter for Nodes {
    fn value(&self) -> ParameterValue {
        ParameterValue::Long(self.0 as i64)
    }

    fn id(&self) -> u32 {
        CPXPARAM_MIP_Limits_Nodes
    }
}
