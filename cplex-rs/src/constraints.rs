use std::ffi::c_char;

use crate::VariableId;

#[derive(Copy, Clone, Debug)]
pub enum ConstraintType {
    LessThanEq,
    Eq,
    GreaterThanEq,
}

impl ConstraintType {
    pub(crate) fn into_raw(self) -> c_char {
        match self {
            ConstraintType::LessThanEq => 'L' as c_char,
            ConstraintType::Eq => 'E' as c_char,
            ConstraintType::GreaterThanEq => 'G' as c_char,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Constraint {
    weights: Vec<(VariableId, f64)>,
    type_: ConstraintType,
    rhs: f64,
    name: String,
}

impl Constraint {
    pub fn new<S, F>(
        ty: ConstraintType,
        rhs: F,
        name: S,
        vars: Vec<(VariableId, f64)>,
    ) -> Constraint
    where
        String: From<S>,
        f64: From<F>,
    {
        let rhs = rhs.into();
        let name = name.into();
        Constraint {
            weights: vars,
            type_: ty,
            rhs,
            name,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn weights(&self) -> &[(VariableId, f64)] {
        &self.weights
    }

    pub fn rhs(&self) -> f64 {
        self.rhs
    }

    pub fn type_(&self) -> ConstraintType {
        self.type_
    }
}
