use crate::VariableId;

#[derive(Clone, Debug)]
pub struct Solution {
    objective_value: f64,
    variable_values: Vec<f64>,
}

impl Solution {
    pub(crate) fn new(variable_values: Vec<f64>, objective_value: f64) -> Self {
        Self {
            objective_value,
            variable_values,
        }
    }

    pub fn objective_value(&self) -> f64 {
        self.objective_value
    }

    pub fn variable_values(&self) -> &[f64] {
        &self.variable_values
    }

    pub fn variable_value(&self, v: VariableId) -> f64 {
        self.variable_values[v.0]
    }
}
