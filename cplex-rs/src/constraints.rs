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
    name: Option<String>,
}

impl Constraint {
    pub fn new(
        ty: ConstraintType,
        rhs: f64,
        name: Option<String>,
        vars: Vec<(VariableId, f64)>,
    ) -> Constraint {
        Constraint {
            weights: vars,
            type_: ty,
            rhs,
            name,
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
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
