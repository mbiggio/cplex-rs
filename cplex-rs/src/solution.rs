use crate::variables::{Variable, VariableType};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VariableValue {
    Continuous(f64),
    Binary(bool),
    Integer(i32),
    SemiContinuous(f64),
    SemiInteger(i32),
}

#[derive(Clone, Debug)]
pub struct Solution {
    objective_value: f64,
    variable_values: Vec<VariableValue>,
    variables: Vec<Variable>,
}

impl Solution {
    pub(crate) fn new(
        variables: Vec<Variable>,
        variable_values: Vec<f64>,
        objective_value: f64,
        tolerance: f64,
    ) -> Self {
        Self {
            objective_value,
            variable_values: variable_values
                .iter()
                .zip(variables.iter())
                .map(|(&x, v)| match v.type_() {
                    VariableType::Binary => {
                        VariableValue::Binary(x <= 1.0 + tolerance && x >= 1.0 - tolerance)
                    }
                    VariableType::Continuous => VariableValue::Continuous(x),
                    VariableType::Integer => VariableValue::Integer(x as i32),
                    VariableType::SemiContinuous => VariableValue::SemiContinuous(x),
                    VariableType::SemiInteger => VariableValue::SemiInteger(x as i32),
                })
                .collect::<Vec<VariableValue>>(),
            variables,
        }
    }

    pub fn objective_value(&self) -> f64 {
        self.objective_value
    }

    pub fn variables(&self) -> &[Variable] {
        &self.variables
    }

    pub fn variable_valuess(&self) -> &[VariableValue] {
        &self.variable_values
    }
}
