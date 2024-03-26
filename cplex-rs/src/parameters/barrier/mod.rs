pub mod limits;

use ffi::{
    CPXPARAM_Barrier_Algorithm, CPXPARAM_Barrier_ColNonzeros, CPXPARAM_Barrier_ConvergeTol,
    CPXPARAM_Barrier_Crossover, CPXPARAM_Barrier_Display, CPXPARAM_Barrier_QCPConvergeTol,
    CPXPARAM_Barrier_StartAlg,
};

use crate::errors::{self, Result};
use crate::parameters::{private, Parameter, ParameterValue};

impl private::Parameter for Algorithm {}
impl private::Parameter for ColNonzeros {}
impl private::Parameter for Crossover {}
impl private::Parameter for Display {}
impl private::Parameter for ConvergeTol {}
impl private::Parameter for QCPConvergeTol {}
impl private::Parameter for StartAlg {}

/// Barrier algorithm.
/// <https://www.ibm.com/docs/en/icos/22.1.1?topic=parameters-barrier-algorithm>
#[derive(Copy, Clone, Debug)]
pub enum Algorithm {
    Default,
    InfeasibilityEstimateStart,
    InfeasibilityConstantStart,
    StandardBarrier,
}

impl Parameter for Algorithm {
    fn value(&self) -> ParameterValue {
        ParameterValue::Integer(match self {
            Self::Default => 0,
            Self::InfeasibilityEstimateStart => 1,
            Self::InfeasibilityConstantStart => 2,
            Self::StandardBarrier => 3,
        })
    }

    fn id(&self) -> u32 {
        CPXPARAM_Barrier_Algorithm
    }
}

/// Barrier column nonzeros.
/// <https://www.ibm.com/docs/en/icos/22.1.1?topic=parameters-barrier-column-nonzeros>
#[derive(Copy, Clone, Debug)]
pub struct ColNonzeros(u32);

impl Parameter for ColNonzeros {
    fn value(&self) -> ParameterValue {
        ParameterValue::Integer(self.0 as i32)
    }

    fn id(&self) -> u32 {
        CPXPARAM_Barrier_ColNonzeros
    }
}

/// Barrier crossover algorithm.
/// <https://www.ibm.com/docs/en/icos/22.1.1?topic=parameters-barrier-crossover-algorithm>
#[derive(Copy, Clone, Debug)]
pub enum Crossover {
    Automatic,
    PrimalCrossover,
    DualCrossover,
}

impl Parameter for Crossover {
    fn value(&self) -> ParameterValue {
        ParameterValue::Integer(match self {
            Self::Automatic => 0,
            Self::PrimalCrossover => 1,
            Self::DualCrossover => 2,
        })
    }

    fn id(&self) -> u32 {
        CPXPARAM_Barrier_Crossover
    }
}

/// Barrier display information.
/// <https://www.ibm.com/docs/en/icos/22.1.1?topic=parameters-barrier-display-information>
#[derive(Copy, Clone, Debug)]
pub enum Display {
    None,
    NormalSetupAndIteration,
    Diagnostic,
}

impl Parameter for Display {
    fn value(&self) -> ParameterValue {
        ParameterValue::Integer(match self {
            Self::None => 0,
            Self::NormalSetupAndIteration => 1,
            Self::Diagnostic => 2,
        })
    }

    fn id(&self) -> u32 {
        CPXPARAM_Barrier_Display
    }
}

/// Convergence tolerance for LP and QP problems.
/// <https://www.ibm.com/docs/en/icos/22.1.1?topic=parameters-convergence-tolerance-lp-qp-problems>
#[derive(Copy, Clone, Debug)]
pub struct ConvergeTol(f64);

impl ConvergeTol {
    pub fn new(value: f64) -> Result<Self> {
        if value < 0.0 {
            return Err(errors::Input::from_message(
                "CPXPARAM_Barrier_ConvergeTol cannot be < 0.0".to_string(),
            )
            .into());
        }
        Ok(Self(value))
    }
}

impl Parameter for ConvergeTol {
    fn value(&self) -> ParameterValue {
        ParameterValue::Double(self.0)
    }

    fn id(&self) -> u32 {
        CPXPARAM_Barrier_ConvergeTol
    }
}

/// Convergence tolerance for QC problems.
/// <https://www.ibm.com/docs/en/icos/22.1.1?topic=parameters-convergence-tolerance-qc-problems>
#[derive(Copy, Clone, Debug)]
pub struct QCPConvergeTol(f64);

impl QCPConvergeTol {
    pub fn new(value: f64) -> Result<Self> {
        if value < 0.0 {
            return Err(errors::Input::from_message(
                "CPXPARAM_Barrier_QCPConvergeTol cannot be < 0.0".to_string(),
            )
            .into());
        }
        Ok(Self(value))
    }
}

impl Parameter for QCPConvergeTol {
    fn value(&self) -> ParameterValue {
        ParameterValue::Double(self.0)
    }

    fn id(&self) -> u32 {
        CPXPARAM_Barrier_QCPConvergeTol
    }
}

/// Barrier starting point algorithm.
/// <https://www.ibm.com/docs/en/icos/22.1.1?topic=parameters-barrier-starting-point-algorithm>
#[derive(Copy, Clone, Debug)]
pub enum StartAlg {
    DualIs0,
    EstimateDual,
    AverageOfPrimalEstimateDualIs0,
    AverageOfPrimalEstimateEstimateDual,
}

impl Parameter for StartAlg {
    fn value(&self) -> ParameterValue {
        ParameterValue::Integer(match self {
            Self::DualIs0 => 1,
            Self::EstimateDual => 2,
            Self::AverageOfPrimalEstimateDualIs0 => 3,
            Self::AverageOfPrimalEstimateEstimateDual => 4,
        })
    }

    fn id(&self) -> u32 {
        CPXPARAM_Barrier_StartAlg
    }
}
