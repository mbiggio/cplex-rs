use ffi::{CPXPARAM_Read_APIEncoding, CPXPARAM_Read_DataCheck};

use crate::parameters::{private, Parameter, ParameterValue};

impl private::Parameter for APIEncoding {}
impl private::Parameter for DataCheck {}

/// API string encoding switch.
/// <https://www.ibm.com/docs/en/icos/22.1.1?topic=parameters-api-string-encoding-switch>
#[derive(Copy, Clone, Debug)]
pub struct APIEncoding(pub &'static str);

impl Parameter for APIEncoding {
    fn value(&self) -> ParameterValue {
        ParameterValue::String(self.0)
    }

    fn id(&self) -> u32 {
        CPXPARAM_Read_APIEncoding
    }
}

/// Data consistency checking and modeling assistance.
/// <https://www.ibm.com/docs/en/icos/20.1.0?topic=parameters-data-consistency-checking-modeling-assistance>
#[derive(Copy, Clone, Debug)]
pub enum DataCheck {
    Off,
    Warning,
    Assist,
}

impl Parameter for DataCheck {
    fn value(&self) -> ParameterValue {
        ParameterValue::Integer(match self {
            Self::Off => 0,
            Self::Warning => 1,
            Self::Assist => 2,
        })
    }

    fn id(&self) -> u32 {
        CPXPARAM_Read_DataCheck
    }
}
