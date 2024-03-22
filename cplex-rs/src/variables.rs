use ffi::{CPX_BINARY, CPX_CONTINUOUS, CPX_INTEGER, CPX_SEMICONT, CPX_SEMIINT};

#[derive(Copy, Clone, Debug)]
pub enum VariableType {
    Continuous,
    Binary,
    Integer,
    SemiContinuous,
    SemiInteger,
}

impl VariableType {
    pub(crate) fn into_raw(self) -> u8 {
        match self {
            VariableType::Continuous => CPX_CONTINUOUS,
            VariableType::Binary => CPX_BINARY,
            VariableType::Integer => CPX_INTEGER,
            VariableType::SemiContinuous => CPX_SEMICONT,
            VariableType::SemiInteger => CPX_SEMIINT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Variable {
    type_: VariableType,
    weight: f64,
    lower_bound: f64,
    upper_bound: f64,
    name: String,
}

impl Variable {
    pub fn new<S>(ty: VariableType, obj: f64, lb: f64, ub: f64, name: S) -> Variable
    where
        String: From<S>,
    {
        let name = name.into();
        Variable {
            type_: ty,
            weight: obj,
            lower_bound: lb,
            upper_bound: ub,
            name,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn upper_bound(&self) -> f64 {
        self.upper_bound
    }

    pub fn lower_bound(&self) -> f64 {
        self.lower_bound
    }

    pub fn weight(&self) -> f64 {
        self.weight
    }

    pub fn type_(&self) -> VariableType {
        self.type_
    }
}
