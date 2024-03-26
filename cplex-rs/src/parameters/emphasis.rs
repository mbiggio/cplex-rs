use ffi::CPXPARAM_Emphasis_MIP;

use super::{private, Parameter, ParameterValue};

impl private::Parameter for MIP {}

/// MIP emphasis switch.
/// <https://www.ibm.com/docs/en/icos/22.1.1?topic=parameters-mip-emphasis-switch>
#[derive(Copy, Clone, Debug)]
pub enum MIP {
    Balanced,
    Feasibility,
    Optimality,
    BestBound,
    HiddenFeas,
    Heuristic,
}

impl Parameter for MIP {
    fn value(&self) -> ParameterValue {
        ParameterValue::Integer(match self {
            Self::Balanced => 0,
            Self::Feasibility => 1,
            Self::Optimality => 2,
            Self::BestBound => 3,
            Self::HiddenFeas => 4,
            Self::Heuristic => 5,
        })
    }

    fn id(&self) -> u32 {
        CPXPARAM_Emphasis_MIP
    }
}
