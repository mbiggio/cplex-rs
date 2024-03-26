use ffi::{CPXPARAM_Preprocessing_Aggregator, CPXPARAM_Preprocessing_Fill};

use crate::parameters::{private, Parameter, ParameterValue};

impl private::Parameter for Fill {}
impl private::Parameter for Aggregator {}

/// Preprocessing aggregator fill.
/// <https://www.ibm.com/docs/en/icos/22.1.1?topic=parameters-preprocessing-aggregator-fill>
#[derive(Copy, Clone, Debug)]
pub struct Fill(pub u32);

impl Parameter for Fill {
    fn value(&self) -> ParameterValue {
        ParameterValue::Integer(self.0 as i32)
    }

    fn id(&self) -> u32 {
        CPXPARAM_Preprocessing_Fill
    }
}

/// Preprocessing aggregator application limit
/// <https://www.ibm.com/docs/en/icos/22.1.1?topic=parameters-preprocessing-aggregator-application-limit>
#[derive(Copy, Clone, Debug)]
pub enum Aggregator {
    Automatic,
    NbOfTimesToApply(u32),
}

impl Parameter for Aggregator {
    fn value(&self) -> ParameterValue {
        ParameterValue::Integer(match self {
            Self::Automatic => -1,
            &Self::NbOfTimesToApply(times) => times as i32,
        })
    }

    fn id(&self) -> u32 {
        CPXPARAM_Preprocessing_Aggregator
    }
}
