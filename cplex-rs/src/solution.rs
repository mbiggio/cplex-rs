use crate::variables::Variable;

#[derive(Clone, Debug)]
pub struct Solution {
    objective_value: f64,
    variable_values: Vec<f64>,
    variables: Vec<Variable>,
}

impl Solution {
    pub(crate) fn new(
        variables: Vec<Variable>,
        variable_values: Vec<f64>,
        objective_value: f64,
    ) -> Self {
        Self {
            objective_value,
            variable_values,
            variables,
        }
    }

    pub fn objective_value(&self) -> f64 {
        self.objective_value
    }

    pub fn variables(&self) -> &[Variable] {
        &self.variables
    }

    pub fn variable_values(&self) -> &[f64] {
        &self.variable_values
    }
}
