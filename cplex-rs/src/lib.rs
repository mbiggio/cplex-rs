mod error;
mod parameters;

use error::Result;
pub use ffi;
use ffi::{
    cpxenv, cpxlp, CPXaddlazyconstraints, CPXaddmipstarts, CPXaddrows, CPXchgobj, CPXchgobjsen,
    CPXchgprobtype, CPXcloseCPLEX, CPXcreateprob, CPXfreeprob, CPXgetdblparam, CPXgetnumcols,
    CPXgetobjval, CPXgetx, CPXlpopt, CPXmipopt, CPXnewcols, CPXopenCPLEX, CPXsetdblparam,
    CPXsetintparam, CPXwriteprob, CPX_PARAM_EPINT,
};
use parameters::{Parameter, ParameterType};

use std::ffi::{c_char, c_double, c_int, CString};

pub const INFINITY: f64 = 1.0E+20;
pub const EPINT: c_double = 1e-5;

pub struct Env {
    inner: *mut cpxenv,
}

impl Env {
    pub fn new() -> Result<Env> {
        let mut status = 0;
        let env = unsafe { CPXopenCPLEX(&mut status) };
        if env.is_null() {
            Err(error::Cplex::env_error(status).into())
        } else {
            Ok(Env { inner: env })
        }
    }

    pub fn set_param(&mut self, p: Parameter) -> Result<()> {
        let status = match p.param_type() {
            ParameterType::Integer(i) => unsafe { CPXsetintparam(self.inner, p.to_id(), i) },
            ParameterType::Double(d) => unsafe { CPXsetdblparam(self.inner, p.to_id(), d) },
        };

        if status != 0 {
            Err(error::Cplex::from_code(self.inner, status).into())
        } else {
            Ok(())
        }
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        unsafe {
            assert_eq!(CPXcloseCPLEX(&mut self.inner), 0);
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

#[derive(Clone, Copy, Debug)]
pub struct VariableId(usize);

#[derive(Clone, Copy, Debug)]
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
        let name =
            CString::new(name.as_ref()).map_err(|e| error::Input::from_message(e.to_string()))?;
        let inner = unsafe { CPXcreateprob(env.inner, &mut status, name.as_ptr()) };
        if inner.is_null() {
            Err(error::Cplex::from_code(env.inner, status).into())
        } else {
            Ok(Problem {
                inner,
                env,
                variables: vec![],
                constraints: vec![],
            })
        }
    }

    /// Add a variable to the problem. The Variable is **moved** into
    /// the problem. At this time, it is not possible to get a
    /// reference to it back.
    ///
    /// The column index for the Variable is returned.
    pub fn add_variable(&mut self, var: Variable) -> Result<VariableId> {
        let name = CString::new(var.name.as_bytes())
            .map_err(|e| error::Input::from_message(e.to_string()))?;
        let status = unsafe {
            CPXnewcols(
                self.env.inner,
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
            Err(error::Cplex::from_code(self.env.inner, status).into())
        } else {
            let index = unsafe { CPXgetnumcols(self.env.inner, self.inner) } as usize - 1;
            assert_eq!(self.variables.len(), index);
            self.variables.push(var);
            Ok(VariableId(index))
        }
    }

    /// Add a constraint to the problem.
    ///
    /// The row index for the constraint is returned.
    pub fn add_constraint(&mut self, con: Constraint) -> Result<ConstraintId> {
        let (ind, val): (Vec<c_int>, Vec<f64>) = con
            .vars
            .iter()
            .filter(|(_, weight)| *weight != 0.0)
            .map(|(var_id, weight)| (var_id.0 as c_int, weight))
            .unzip();
        let nz = val.len() as c_int;
        let name = CString::new(con.name.as_bytes())
            .map_err(|e| error::Input::from_message(e.to_string()))?;
        let status = unsafe {
            CPXaddrows(
                self.env.inner,
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
            Err(error::Cplex::from_code(self.env.inner, status).into())
        } else {
            let index = self.constraints.len();
            self.constraints.push(con);
            Ok(ConstraintId(index))
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
        let name = CString::new(con.name.as_bytes())
            .map_err(|e| error::Input::from_message(e.to_string()))?;
        let status = unsafe {
            CPXaddlazyconstraints(
                self.env.inner,
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
            Err(error::Cplex::from_code(self.env.inner, status).into())
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
                self.env.inner,
                self.inner,
                ind.len() as c_int,
                ind.as_ptr(),
                val.as_ptr(),
            )
        };

        if status != 0 {
            Err(error::Cplex::from_code(self.env.inner, status).into())
        } else {
            self.set_objective_type(ty)
        }
    }

    /// Change the objective type. Default: `ObjectiveType::Minimize`.
    ///
    /// It is recommended to use this in conjunction with objective
    /// coefficients set by the `var!` macro rather than using
    /// `set_objective`.
    pub fn set_objective_type(&mut self, ty: ObjectiveType) -> Result<()> {
        let status = unsafe { CPXchgobjsen(self.env.inner, self.inner, ty.into_raw()) };
        if status != 0 {
            Err(error::Cplex::from_code(self.env.inner, status).into())
        } else {
            Ok(())
        }
    }

    /// Write the problem to a file named `name`. At this time, it is
    /// not possible to use a `Write` object instead, as this calls C
    /// code directly.
    pub fn write<S>(&self, name: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        let name =
            CString::new(name.as_ref()).map_err(|e| error::Input::from_message(e.to_string()))?;
        let status =
            unsafe { CPXwriteprob(self.env.inner, self.inner, name.as_ptr(), std::ptr::null()) };

        if status != 0 {
            Err(error::Cplex::from_code(self.env.inner, status).into())
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
                self.env.inner,
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
            Err(error::Cplex::from_code(self.env.inner, status).into())
        } else {
            Ok(())
        }
    }

    /// Solve the Problem, returning a `Solution` object with the
    /// result.
    pub fn solve_as(&mut self, pt: ProblemType) -> Result<Solution> {
        if pt == ProblemType::Linear {
            let status = unsafe { CPXchgprobtype(self.env.inner, self.inner, 0) };

            if status != 0 {
                return Err(error::Cplex::from_code(self.env.inner, status).into());
            }
        }
        let status = match pt {
            ProblemType::MixedInteger => unsafe { CPXmipopt(self.env.inner, self.inner) },
            ProblemType::Linear => unsafe { CPXlpopt(self.env.inner, self.inner) },
        };
        if status != 0 {
            return Err(error::Cplex::from_code(self.env.inner, status).into());
        }

        let mut objval: f64 = 0.0;
        let status = unsafe { CPXgetobjval(self.env.inner, self.inner, &mut objval) };
        if status != 0 {
            return Err(error::Cplex::from_code(self.env.inner, status).into());
        }

        let mut xs = vec![0f64; self.variables.len()];
        let status = unsafe {
            CPXgetx(
                self.env.inner,
                self.inner,
                xs.as_mut_ptr(),
                0,
                self.variables.len() as c_int - 1,
            )
        };
        if status != 0 {
            return Err(error::Cplex::from_code(self.env.inner, status).into());
        }
        let mut eps = EPINT;
        unsafe { CPXgetdblparam(self.env.inner, CPX_PARAM_EPINT as i32, &mut eps) };
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

    /// Solve the problem as a Mixed Integer Program
    pub fn solve(&mut self) -> Result<Solution> {
        self.solve_as(ProblemType::MixedInteger)
    }
}

impl<'a> Drop for Problem<'a> {
    fn drop(&mut self) {
        unsafe {
            assert_eq!(CPXfreeprob(self.env.inner, &mut self.inner), 0);
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

        let _c0 = problem.add_constraint(Constraint::new(
            ConstraintType::LessThanEq,
            20.0,
            "c0",
            vec![(x0, -1.0), (x1, 1.0), (x2, 1.0), (x3, 10.0)],
        ));

        let _c1 = problem.add_constraint(Constraint::new(
            ConstraintType::LessThanEq,
            30.0,
            "c1",
            vec![(x0, 1.0), (x1, -3.0), (x2, 1.0)],
        ));

        let _c2 = problem.add_constraint(Constraint::new(
            ConstraintType::Eq,
            0.0,
            "c2",
            vec![(x1, 1.0), (x3, -3.5)],
        ));

        problem.set_objective_type(ObjectiveType::Maximize).unwrap();

        let solution = problem.solve_as(ProblemType::MixedInteger).unwrap();

        assert_eq!(solution.objective, 122.5);
    }
}
