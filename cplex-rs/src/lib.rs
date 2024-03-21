mod error;
pub mod parameters;

use error::Result;
pub use ffi;
use ffi::{
    cpxenv, cpxlp, CPXaddlazyconstraints, CPXaddmipstarts, CPXaddrows, CPXchgobj, CPXchgobjsen,
    CPXchgprobtype, CPXcloseCPLEX, CPXcreateprob, CPXfreeprob, CPXgetdblparam, CPXgetnumcols,
    CPXgetobjval, CPXgetx, CPXlpopt, CPXmipopt, CPXnewcols, CPXopenCPLEX, CPXsetdblparam,
    CPXsetintparam, CPXwriteprob, CPX_PARAM_EPINT,
};
use log::error;
use parameters::{Parameter, ParameterValue};

use std::{
    ffi::{c_char, c_double, c_int, CString},
    iter,
};

pub const INFINITY: f64 = 1.0E+20;
pub const EPINT: c_double = 1e-5;

pub struct Env(*mut cpxenv);

impl Env {
    pub fn new() -> Result<Env> {
        let mut status = 0;
        let env = unsafe { CPXopenCPLEX(&mut status) };
        if env.is_null() {
            Err(error::Cplex::env_error(status).into())
        } else {
            Ok(Env(env))
        }
    }

    pub fn set_parameter<P: Parameter>(&mut self, p: P) -> Result<()> {
        let status = match p.value() {
            ParameterValue::Integer(i) => unsafe { CPXsetintparam(self.0, p.id() as i32, i) },
            ParameterValue::Double(d) => unsafe { CPXsetdblparam(self.0, p.id() as i32, d) },
        };

        if status != 0 {
            Err(
                error::Cplex::from_code(self.0, status, Some("Failure in setting parameters"))
                    .into(),
            )
        } else {
            Ok(())
        }
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        unsafe {
            let status = CPXcloseCPLEX(&mut self.0);
            if status != 0 {
                error!("Unable to close CPLEX context, got status: '{}'", status)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Variable {
    ty: VariableType,
    obj: f64,
    lb: f64,
    ub: f64,
    name: String,
}

impl Variable {
    pub fn new<S>(ty: VariableType, obj: f64, lb: f64, ub: f64, name: S) -> Variable
    where
        S: Into<String>,
    {
        Variable {
            ty,
            obj,
            lb,
            ub,
            name: name.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VariableId(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConstraintId(usize);

#[derive(Clone, Debug)]
pub struct Constraint {
    vars: Vec<(VariableId, f64)>,
    ty: ConstraintType,
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
        S: Into<String>,
        F: Into<f64>,
    {
        Constraint {
            vars,
            ty,
            rhs: rhs.into(),
            name: name.into(),
        }
    }
}

pub struct Problem<'a> {
    inner: *mut cpxlp,
    env: &'a Env,
    variables: Vec<Variable>,
    constraints: Vec<Constraint>,
}

#[derive(Clone, Debug)]
pub struct Solution {
    pub objective: f64,
    pub variables: Vec<VariableValue>,
}

#[derive(Copy, Clone, Debug)]
pub enum ObjectiveType {
    Maximize,
    Minimize,
}

#[derive(Copy, Clone, Debug)]
pub enum VariableType {
    Continuous,
    Binary,
    Integer,
    /// A variable bounded by `[lb, ub]` or equal to 0.
    SemiContinuous,
    /// An integer variable bounded by `[lb, ub]` or equal to 0.
    SemiInteger,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VariableValue {
    Continuous(f64),
    Binary(bool),
    Integer(c_int),
    SemiContinuous(f64),
    SemiInteger(c_int),
}

#[derive(Copy, Clone, Debug)]
pub enum ConstraintType {
    LessThanEq,
    Eq,
    GreaterThanEq,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ProblemType {
    Linear,
    MixedInteger,
}

impl VariableType {
    fn into_raw(self) -> c_char {
        match self {
            VariableType::Continuous => 'C' as c_char,
            VariableType::Binary => 'B' as c_char,
            VariableType::Integer => 'I' as c_char,
            VariableType::SemiContinuous => 'S' as c_char,
            VariableType::SemiInteger => 'N' as c_char,
        }
    }
}

impl ConstraintType {
    fn into_raw(self) -> c_char {
        match self {
            ConstraintType::LessThanEq => 'L' as c_char,
            ConstraintType::Eq => 'E' as c_char,
            ConstraintType::GreaterThanEq => 'G' as c_char,
        }
    }
}

impl ObjectiveType {
    fn into_raw(self) -> c_int {
        match self {
            ObjectiveType::Minimize => 1 as c_int,
            ObjectiveType::Maximize => -1 as c_int,
        }
    }
}

impl<'a> Problem<'a> {
    pub fn new<S>(env: &'a Env, name: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        let mut status = 0;
        let name = CString::new(name.as_ref())
            .map_err(|e| error::Input::from_message(e.to_string(), None))?;
        let inner = unsafe { CPXcreateprob(env.0, &mut status, name.as_ptr()) };
        if inner.is_null() {
            Err(error::Cplex::from_code(env.0, status, Some("Failure in problem creation")).into())
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
        let name = CString::new(var.name.as_bytes()).map_err(|e| {
            error::Input::from_message(e.to_string(), Some("Failure in adding variable"))
        })?;
        let status = unsafe {
            CPXnewcols(
                self.env.0,
                self.inner,
                1,
                &var.obj,
                &var.lb,
                &var.ub,
                &var.ty.into_raw(),
                &mut (name.as_ptr() as *mut _),
            )
        };

        if status != 0 {
            Err(
                error::Cplex::from_code(self.env.0, status, Some("Failure in adding variable"))
                    .into(),
            )
        } else {
            let index = unsafe { CPXgetnumcols(self.env.0, self.inner) } as usize - 1;
            assert_eq!(self.variables.len(), index);
            self.variables.push(var);
            Ok(VariableId(index))
        }
    }

    pub fn add_variables(&mut self, vars: Vec<Variable>) -> Result<Vec<VariableId>> {
        let names = vars
            .iter()
            .map(|v| {
                CString::new(v.name.as_bytes()).map_err(|e| {
                    error::Input::from_message(e.to_string(), Some("Failure in adding variable"))
                        .into()
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let mut name_ptrs = names
            .iter()
            .map(|n| n.as_ptr() as *mut _)
            .collect::<Vec<_>>();

        let objs = vars.iter().map(|v| v.obj).collect::<Vec<_>>();
        let lbs = vars.iter().map(|v| v.lb).collect::<Vec<_>>();
        let ubs = vars.iter().map(|v| v.ub).collect::<Vec<_>>();
        let types = vars.iter().map(|v| v.ty.into_raw()).collect::<Vec<_>>();

        let status = unsafe {
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
        };

        if status != 0 {
            Err(
                error::Cplex::from_code(self.env.0, status, Some("Failure in adding variable"))
                    .into(),
            )
        } else {
            let indices: Vec<VariableId> = vars
                .iter()
                .enumerate()
                .map(|(idx, _)| VariableId(idx + self.variables.len()))
                .collect();
            self.variables.extend(vars);
            Ok(indices)
        }
    }

    /// Add a constraint to the problem.
    ///
    /// The id for the constraint is returned.
    pub fn add_constraint(&mut self, con: Constraint) -> Result<ConstraintId> {
        let (ind, val): (Vec<c_int>, Vec<f64>) = con
            .vars
            .iter()
            .filter(|(_, weight)| *weight != 0.0)
            .map(|(var_id, weight)| (var_id.0 as c_int, weight))
            .unzip();
        let nz = val.len() as c_int;
        let name = CString::new(con.name.as_bytes()).map_err(|e| {
            error::Input::from_message(e.to_string(), Some("Failure in adding constraint"))
        })?;
        let status = unsafe {
            CPXaddrows(
                self.env.0,
                self.inner,
                0,
                1,
                nz,
                &con.rhs,
                &con.ty.into_raw(),
                &0,
                ind.as_ptr(),
                val.as_ptr(),
                std::ptr::null_mut(),
                &mut (name.as_ptr() as *mut _),
            )
        };

        if status != 0 {
            Err(
                error::Cplex::from_code(self.env.0, status, Some("Failure in adding constraint"))
                    .into(),
            )
        } else {
            let index = self.constraints.len();
            self.constraints.push(con);
            Ok(ConstraintId(index))
        }
    }

    pub fn add_constraints(&mut self, con: Vec<Constraint>) -> Result<Vec<ConstraintId>> {
        if con.is_empty() {
            return Err(error::Input::from_message(
                "Called add_constraints with 0 constaints".to_owned(),
                None,
            )
            .into());
        }
        let beg = iter::once(0)
            .chain(con[..con.len() - 1].iter().map(|c| c.vars.len()))
            .scan(0, |state, x| {
                *state += x;
                Some(*state as i32)
            })
            .collect::<Vec<_>>();

        let (ind, val): (Vec<c_int>, Vec<f64>) = con
            .iter()
            .flat_map(|c| c.vars.iter())
            .filter(|(_, weight)| *weight != 0.0)
            .map(|(var_id, weight)| (var_id.0 as c_int, weight))
            .unzip();

        let nz = val.len() as c_int;
        let names = con
            .iter()
            .map(|c| {
                CString::new(c.name.as_bytes()).map_err(|e| {
                    error::Input::from_message(e.to_string(), Some("Failure in adding constraint"))
                        .into()
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let mut name_ptrs = names
            .iter()
            .map(|n| n.as_ptr() as *mut _)
            .collect::<Vec<_>>();

        let rhss = con.iter().map(|c| c.rhs).collect::<Vec<_>>();
        let senses = con.iter().map(|c| c.ty.into_raw()).collect::<Vec<_>>();

        let status = unsafe {
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
        };

        if status != 0 {
            Err(
                error::Cplex::from_code(self.env.0, status, Some("Failure in adding constraint"))
                    .into(),
            )
        } else {
            let indices = con
                .iter()
                .enumerate()
                .map(|(idx, _)| ConstraintId(idx + self.constraints.len()))
                .collect();
            self.constraints.extend(con);
            Ok(indices)
        }
    }

    /// Adds a lazy constraint to the problem.
    ///
    /// Returns the index of the constraint. Unclear if this has any value whatsoever.
    pub fn add_lazy_constraint(&mut self, con: Constraint) -> Result<ConstraintId> {
        let (ind, val): (Vec<c_int>, Vec<f64>) = con
            .vars
            .iter()
            .filter(|(_, weight)| *weight != 0.0)
            .map(|(var_id, weight)| (var_id.0 as c_int, weight))
            .unzip();
        let nz = val.len() as c_int;
        let name = CString::new(con.name.as_bytes()).map_err(|e| {
            error::Input::from_message(e.to_string(), Some("Failure in adding lazy constraint"))
        })?;
        let status = unsafe {
            CPXaddlazyconstraints(
                self.env.0,
                self.inner,
                1,
                nz,
                &con.rhs,
                &con.ty.into_raw(),
                &0,
                ind.as_ptr(),
                val.as_ptr(),
                &mut (name.as_ptr() as *mut _),
            )
        };

        if status != 0 {
            Err(error::Cplex::from_code(
                self.env.0,
                status,
                Some("Failure in adding lazy constraint"),
            )
            .into())
        } else {
            let index = self.constraints.len();
            self.constraints.push(con);
            Ok(ConstraintId(index))
        }
    }

    /// Set the objective coefficients.
    pub fn set_objective(&mut self, ty: ObjectiveType, obj: Vec<(VariableId, f64)>) -> Result<()> {
        let (ind, val): (Vec<c_int>, Vec<f64>) = obj
            .into_iter()
            .map(|(var_id, weight)| (var_id.0 as c_int, weight))
            .unzip();
        let status = unsafe {
            CPXchgobj(
                self.env.0,
                self.inner,
                ind.len() as c_int,
                ind.as_ptr(),
                val.as_ptr(),
            )
        };

        if status != 0 {
            Err(
                error::Cplex::from_code(self.env.0, status, Some("Failure in setting objective"))
                    .into(),
            )
        } else {
            self.set_objective_type(ty)
        }
    }

    /// Change the objective type. Default: `ObjectiveType::Minimize`.
    pub fn set_objective_type(&mut self, ty: ObjectiveType) -> Result<()> {
        let status = unsafe { CPXchgobjsen(self.env.0, self.inner, ty.into_raw()) };
        if status != 0 {
            Err(error::Cplex::from_code(
                self.env.0,
                status,
                Some("Failure in setting objective type"),
            )
            .into())
        } else {
            Ok(())
        }
    }

    /// Write the problem to a file named `name`.
    pub fn write<S>(&self, name: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        let name = CString::new(name.as_ref()).map_err(|e| {
            error::Input::from_message(e.to_string(), Some("Failure in writing problem file"))
        })?;
        let status =
            unsafe { CPXwriteprob(self.env.0, self.inner, name.as_ptr(), std::ptr::null()) };

        if status != 0 {
            Err(error::Cplex::from_code(
                self.env.0,
                status,
                Some("Failure in writing problem file"),
            )
            .into())
        } else {
            Ok(())
        }
    }

    /// Add an initial solution to the problem.
    ///
    /// `vars` is an array of indices (i.e. the result of `prob.add_variable`) and `values` are
    /// their values.
    pub fn add_initial_soln(&mut self, vars: &[VariableId], values: &[f64]) -> Result<()> {
        assert_eq!(values.len(), vars.len());
        let vars = vars.iter().map(|&u| u.0 as c_int).collect::<Vec<_>>();

        let status = unsafe {
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
        };

        if status != 0 {
            Err(error::Cplex::from_code(
                self.env.0,
                status,
                Some("Failure in adding initial solution"),
            )
            .into())
        } else {
            Ok(())
        }
    }

    /// Solve the Problem, returning a `Solution` object with the
    /// result.
    pub fn solve_as(&mut self, pt: ProblemType) -> Result<Solution> {
        if pt == ProblemType::Linear {
            let status = unsafe { CPXchgprobtype(self.env.0, self.inner, 0) };

            if status != 0 {
                return Err(error::Cplex::from_code(
                    self.env.0,
                    status,
                    Some("Failure in solving problem"),
                )
                .into());
            }
        }
        let status = match pt {
            ProblemType::MixedInteger => unsafe { CPXmipopt(self.env.0, self.inner) },
            ProblemType::Linear => unsafe { CPXlpopt(self.env.0, self.inner) },
        };
        if status != 0 {
            return Err(error::Cplex::from_code(
                self.env.0,
                status,
                Some("Failure in solving problem"),
            )
            .into());
        }

        let mut objval: f64 = 0.0;
        let status = unsafe { CPXgetobjval(self.env.0, self.inner, &mut objval) };
        if status != 0 {
            return Err(error::Cplex::from_code(
                self.env.0,
                status,
                Some("Failure in solving problem"),
            )
            .into());
        }

        let mut xs = vec![0f64; self.variables.len()];
        let status = unsafe {
            CPXgetx(
                self.env.0,
                self.inner,
                xs.as_mut_ptr(),
                0,
                self.variables.len() as c_int - 1,
            )
        };
        if status != 0 {
            return Err(error::Cplex::from_code(
                self.env.0,
                status,
                Some("Failure in solving problem"),
            )
            .into());
        }
        let mut eps = EPINT;
        unsafe { CPXgetdblparam(self.env.0, CPX_PARAM_EPINT as i32, &mut eps) };
        return Ok(Solution {
            objective: objval,
            variables: xs
                .iter()
                .zip(self.variables.iter())
                .map(|(&x, v)| match v.ty {
                    VariableType::Binary => VariableValue::Binary(x <= 1.0 + eps && x >= 1.0 - eps),
                    VariableType::Continuous => VariableValue::Continuous(x),
                    VariableType::Integer => VariableValue::Integer(x as c_int),
                    VariableType::SemiContinuous => VariableValue::SemiContinuous(x),
                    VariableType::SemiInteger => VariableValue::SemiInteger(x as c_int),
                })
                .collect::<Vec<VariableValue>>(),
        });
    }
}

impl<'a> Drop for Problem<'a> {
    fn drop(&mut self) {
        unsafe {
            assert_eq!(CPXfreeprob(self.env.0, &mut self.inner), 0);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mipex1() {
        let env = Env::new().unwrap();
        let mut problem = Problem::new(&env, "mipex1").unwrap();

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

        assert_eq!(solution.objective, 122.5);
    }

    #[test]
    fn mipex1_batch() {
        let env = Env::new().unwrap();
        let mut problem = Problem::new(&env, "mipex1").unwrap();

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

        assert_eq!(solution.objective, 122.5);
    }
}
