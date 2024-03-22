pub mod constants;
pub mod constraints;
pub mod environment;
pub mod errors;
pub mod parameters;
pub mod solution;
pub mod variables;

use constraints::Constraint;
use environment::Environment;
use errors::Result;
pub use ffi;
use ffi::{
    cpxlp, CPXaddmipstarts, CPXaddrows, CPXchgobj, CPXchgobjsen, CPXchgprobtype, CPXcreateprob,
    CPXfreeprob, CPXgetdblparam, CPXgetnumcols, CPXgetobjval, CPXgetx, CPXlpopt, CPXmipopt,
    CPXnewcols, CPXwriteprob, CPXPROB_LP, CPXPROB_MILP, CPX_MAX, CPX_MIN, CPX_PARAM_EPINT,
};

use solution::Solution;
use variables::Variable;

use std::ffi::{c_int, CString};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VariableId(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConstraintId(usize);

pub struct Problem {
    inner: *mut cpxlp,
    env: Environment,
    variables: Vec<Variable>,
    constraints: Vec<Constraint>,
}

#[derive(Copy, Clone, Debug)]
pub enum ObjectiveType {
    Maximize,
    Minimize,
}

impl ObjectiveType {
    fn into_raw(self) -> c_int {
        match self {
            ObjectiveType::Minimize => CPX_MIN as c_int,
            ObjectiveType::Maximize => CPX_MAX as c_int,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ProblemType {
    Linear,
    MixedInteger,
}

impl ProblemType {
    fn into_raw(self) -> c_int {
        match self {
            ProblemType::Linear => CPXPROB_LP as c_int,
            ProblemType::MixedInteger => CPXPROB_MILP as c_int,
        }
    }
}

impl<'a> Problem {
    pub fn new<S>(env: Environment, name: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        let mut status = 0;
        let name =
            CString::new(name.as_ref()).map_err(|e| errors::Input::from_message(e.to_string()))?;
        let inner = unsafe { CPXcreateprob(env.0, &mut status, name.as_ptr()) };
        if inner.is_null() {
            Err(errors::Cplex::from_code(env.0, status).into())
        } else {
            Ok(Problem {
                inner,
                env,
                variables: vec![],
                constraints: vec![],
            })
        }
    }

    /// Add a variable to the problem.
    ///
    /// The id for the Variable is returned.
    pub fn add_variable(&mut self, var: Variable) -> Result<VariableId> {
        let name = CString::new(var.name().as_bytes())
            .map_err(|e| errors::Input::from_message(e.to_string()))?;

        cpx_result!(unsafe {
            CPXnewcols(
                self.env.0,
                self.inner,
                1,
                &var.weight(),
                &var.lower_bound(),
                &var.upper_bound(),
                &var.type_().into_raw() as *const u8 as *const i8,
                &mut (name.as_ptr() as *mut _),
            )
        })?;

        let index = unsafe { CPXgetnumcols(self.env.0, self.inner) } as usize - 1;
        assert_eq!(self.variables.len(), index);
        self.variables.push(var);
        Ok(VariableId(index))
    }

    pub fn add_variables(&mut self, vars: Vec<Variable>) -> Result<Vec<VariableId>> {
        let names = vars
            .iter()
            .map(|v| {
                CString::new(v.name().as_bytes())
                    .map_err(|e| errors::Input::from_message(e.to_string()).into())
            })
            .collect::<Result<Vec<_>>>()?;

        let mut name_ptrs = names
            .iter()
            .map(|n| n.as_ptr() as *mut _)
            .collect::<Vec<_>>();

        let objs = vars.iter().map(|v| v.weight()).collect::<Vec<_>>();
        let lbs = vars.iter().map(|v| v.lower_bound()).collect::<Vec<_>>();
        let ubs = vars.iter().map(|v| v.upper_bound()).collect::<Vec<_>>();
        let types = vars
            .iter()
            .map(|v| v.type_().into_raw() as i8)
            .collect::<Vec<_>>();

        cpx_result!(unsafe {
            CPXnewcols(
                self.env.0,
                self.inner,
                vars.len() as i32,
                objs.as_ptr(),
                lbs.as_ptr(),
                ubs.as_ptr(),
                types.as_ptr(),
                name_ptrs.as_mut_ptr(),
            )
        })?;

        let indices: Vec<VariableId> = vars
            .iter()
            .enumerate()
            .map(|(idx, _)| VariableId(idx + self.variables.len()))
            .collect();
        self.variables.extend(vars);
        Ok(indices)
    }

    /// Add a constraint to the problem.
    ///
    /// The id for the constraint is returned.
    pub fn add_constraint(&mut self, constraint: Constraint) -> Result<ConstraintId> {
        let (ind, val): (Vec<c_int>, Vec<f64>) = constraint
            .weights()
            .iter()
            .filter(|(_, weight)| *weight != 0.0)
            .map(|(var_id, weight)| (var_id.0 as c_int, weight))
            .unzip();
        let nz = val.len() as c_int;
        let name = CString::new(constraint.name().as_bytes())
            .map_err(|e| errors::Input::from_message(e.to_string()))?;
        cpx_result!(unsafe {
            CPXaddrows(
                self.env.0,
                self.inner,
                0,
                1,
                nz,
                &constraint.rhs(),
                &constraint.type_().into_raw(),
                &0,
                ind.as_ptr(),
                val.as_ptr(),
                std::ptr::null_mut(),
                &mut (name.as_ptr() as *mut _),
            )
        })?;

        let index = self.constraints.len();
        self.constraints.push(constraint);
        Ok(ConstraintId(index))
    }

    pub fn add_constraints(&mut self, con: Vec<Constraint>) -> Result<Vec<ConstraintId>> {
        if con.is_empty() {
            return Err(errors::Input::from_message(
                "Called add_constraints with 0 constaints".to_owned(),
            )
            .into());
        }
        let beg = std::iter::once(0)
            .chain(con[..con.len() - 1].iter().map(|c| c.weights().len()))
            .scan(0, |state, x| {
                *state += x;
                Some(*state as i32)
            })
            .collect::<Vec<_>>();

        let (ind, val): (Vec<c_int>, Vec<f64>) = con
            .iter()
            .flat_map(|c| c.weights().iter())
            .filter(|(_, weight)| *weight != 0.0)
            .map(|(var_id, weight)| (var_id.0 as c_int, weight))
            .unzip();

        let nz = val.len() as c_int;
        let names = con
            .iter()
            .map(|c| {
                CString::new(c.name().as_bytes())
                    .map_err(|e| errors::Input::from_message(e.to_string()).into())
            })
            .collect::<Result<Vec<_>>>()?;

        let mut name_ptrs = names
            .iter()
            .map(|n| n.as_ptr() as *mut _)
            .collect::<Vec<_>>();

        let rhss = con.iter().map(|c| c.rhs()).collect::<Vec<_>>();
        let senses = con.iter().map(|c| c.type_().into_raw()).collect::<Vec<_>>();

        cpx_result!(unsafe {
            CPXaddrows(
                self.env.0,
                self.inner,
                0,
                con.len() as i32,
                nz,
                rhss.as_ptr(),
                senses.as_ptr(),
                beg.as_ptr(),
                ind.as_ptr(),
                val.as_ptr(),
                std::ptr::null_mut(),
                name_ptrs.as_mut_ptr(),
            )
        })?;

        let indices = con
            .iter()
            .enumerate()
            .map(|(idx, _)| ConstraintId(idx + self.constraints.len()))
            .collect();
        self.constraints.extend(con);
        Ok(indices)
    }

    /// Set the objective coefficients.
    pub fn set_objective(&mut self, ty: ObjectiveType, obj: Vec<(VariableId, f64)>) -> Result<()> {
        let (ind, val): (Vec<c_int>, Vec<f64>) = obj
            .into_iter()
            .map(|(var_id, weight)| (var_id.0 as c_int, weight))
            .unzip();

        cpx_result!(unsafe {
            CPXchgobj(
                self.env.0,
                self.inner,
                ind.len() as c_int,
                ind.as_ptr(),
                val.as_ptr(),
            )
        })?;

        self.set_objective_type(ty)
    }

    /// Change the objective type. Default: `ObjectiveType::Minimize`.
    pub fn set_objective_type(&mut self, ty: ObjectiveType) -> Result<()> {
        cpx_result!(unsafe { CPXchgobjsen(self.env.0, self.inner, ty.into_raw()) })
    }

    /// Write the problem to a file named `name`.
    pub fn write<S>(&self, name: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        let name =
            CString::new(name.as_ref()).map_err(|e| errors::Input::from_message(e.to_string()))?;

        cpx_result!(unsafe {
            CPXwriteprob(self.env.0, self.inner, name.as_ptr(), std::ptr::null())
        })
    }

    /// Add an initial solution to the problem.
    ///
    /// `vars` is an array of indices (i.e. the result of `prob.add_variable`) and `values` are
    /// their values.
    pub fn add_initial_soln(&mut self, vars: &[VariableId], values: &[f64]) -> Result<()> {
        if values.len() != vars.len() {
            return Err(errors::Input::from_message(
                "number of solution variables and values does not match".to_string(),
            )
            .into());
        }
        let vars = vars.iter().map(|&u| u.0 as c_int).collect::<Vec<_>>();

        cpx_result!(unsafe {
            CPXaddmipstarts(
                self.env.0,
                self.inner,
                1,
                vars.len() as c_int,
                &0,
                vars.as_ptr(),
                values.as_ptr(),
                &0,
                &mut std::ptr::null_mut(),
            )
        })
    }

    /// Solve the Problem, returning a `Solution` object with the
    /// result.
    pub fn solve_as(mut self, pt: ProblemType) -> Result<(Solution, Environment)> {
        cpx_result!(unsafe { CPXchgprobtype(self.env.0, self.inner, pt.into_raw()) })?;

        match pt {
            ProblemType::MixedInteger => cpx_result!(unsafe { CPXmipopt(self.env.0, self.inner) })?,
            ProblemType::Linear => cpx_result!(unsafe { CPXlpopt(self.env.0, self.inner) })?,
        };

        let mut objective_value: f64 = 0.0;
        cpx_result!(unsafe { CPXgetobjval(self.env.0, self.inner, &mut objective_value) })?;

        let mut variable_values = vec![0f64; self.variables.len()];
        cpx_result!(unsafe {
            CPXgetx(
                self.env.0,
                self.inner,
                variable_values.as_mut_ptr(),
                0,
                self.variables.len() as c_int - 1,
            )
        })?;

        let mut tolerance = constants::EPINT;
        cpx_result!(unsafe { CPXgetdblparam(self.env.0, CPX_PARAM_EPINT as i32, &mut tolerance) })?;

        let env = self.env;
        return Ok((
            Solution::new(
                std::mem::take(&mut self.variables),
                variable_values,
                objective_value,
                tolerance,
            ),
            env,
        ));
    }
}

impl Drop for Problem {
    fn drop(&mut self) {
        unsafe {
            assert_eq!(CPXfreeprob(self.env.0, &mut self.inner), 0);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::constraints::ConstraintType;

    use super::*;
    use constants::*;
    use variables::{Variable, VariableType};

    #[test]
    fn mipex1() {
        let env = Environment::new().unwrap();
        let mut problem = Problem::new(env, "mipex1").unwrap();

        let x0 = problem
            .add_variable(Variable::new(
                VariableType::Continuous,
                1.0,
                0.0,
                40.0,
                "x0",
            ))
            .unwrap();

        let x1 = problem
            .add_variable(Variable::new(
                VariableType::Continuous,
                2.0,
                0.0,
                INFINITY,
                "x1",
            ))
            .unwrap();

        let x2 = problem
            .add_variable(Variable::new(
                VariableType::Continuous,
                3.0,
                0.0,
                INFINITY,
                "x2",
            ))
            .unwrap();

        let x3 = problem
            .add_variable(Variable::new(VariableType::Integer, 1.0, 2.0, 3.0, "x3"))
            .unwrap();

        assert_eq!(x0, VariableId(0));
        assert_eq!(x1, VariableId(1));
        assert_eq!(x2, VariableId(2));
        assert_eq!(x3, VariableId(3));

        let c0 = problem
            .add_constraint(Constraint::new(
                ConstraintType::LessThanEq,
                20.0,
                "c0",
                vec![(x0, -1.0), (x1, 1.0), (x2, 1.0), (x3, 10.0)],
            ))
            .unwrap();

        let c1 = problem
            .add_constraint(Constraint::new(
                ConstraintType::LessThanEq,
                30.0,
                "c1",
                vec![(x0, 1.0), (x1, -3.0), (x2, 1.0)],
            ))
            .unwrap();

        let c2 = problem
            .add_constraint(Constraint::new(
                ConstraintType::Eq,
                0.0,
                "c2",
                vec![(x1, 1.0), (x3, -3.5)],
            ))
            .unwrap();

        assert_eq!(c0, ConstraintId(0));
        assert_eq!(c1, ConstraintId(1));
        assert_eq!(c2, ConstraintId(2));

        problem.set_objective_type(ObjectiveType::Maximize).unwrap();

        let solution = problem.solve_as(ProblemType::MixedInteger).unwrap();

        assert_eq!(solution.objective_value(), 122.5);
    }

    #[test]
    fn mipex1_batch() {
        let env = Environment::new().unwrap();
        let mut problem = Problem::new(env, "mipex1").unwrap();

        let vars = problem
            .add_variables(vec![
                Variable::new(VariableType::Continuous, 1.0, 0.0, 40.0, "x0"),
                Variable::new(VariableType::Continuous, 2.0, 0.0, INFINITY, "x1"),
                Variable::new(VariableType::Continuous, 3.0, 0.0, INFINITY, "x2"),
                Variable::new(VariableType::Integer, 1.0, 2.0, 3.0, "x3"),
            ])
            .unwrap();

        assert_eq!(
            vars,
            vec![VariableId(0), VariableId(1), VariableId(2), VariableId(3)]
        );

        let cons = problem
            .add_constraints(vec![
                Constraint::new(
                    ConstraintType::LessThanEq,
                    20.0,
                    "c0",
                    vec![
                        (vars[0], -1.0),
                        (vars[1], 1.0),
                        (vars[2], 1.0),
                        (vars[3], 10.0),
                    ],
                ),
                Constraint::new(
                    ConstraintType::LessThanEq,
                    30.0,
                    "c1",
                    vec![(vars[0], 1.0), (vars[1], -3.0), (vars[2], 1.0)],
                ),
                Constraint::new(
                    ConstraintType::Eq,
                    0.0,
                    "c2",
                    vec![(vars[1], 1.0), (vars[3], -3.5)],
                ),
            ])
            .unwrap();

        assert_eq!(
            cons,
            vec![ConstraintId(0), ConstraintId(1), ConstraintId(2)]
        );

        problem.set_objective_type(ObjectiveType::Maximize).unwrap();

        let solution = problem.solve_as(ProblemType::MixedInteger).unwrap();

        assert_eq!(solution.objective_value(), 122.5);
    }
}
