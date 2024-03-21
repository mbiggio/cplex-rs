#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::approx_constant)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn mipex1() {
        const NUMROWS: usize = 3;
        const NUMCOLS: usize = 4;
        const NUMNZ: usize = 9;

        let mut status = 0;
        let env = unsafe { CPXopenCPLEX(&mut status) };

        if env.is_null() {
            panic!("Could not open CPLEX environment");
        }

        status = unsafe { CPXsetintparam(env, CPXPARAM_ScreenOutput as i32, CPX_ON as i32) };
        if status != 0 {
            panic!("Failure to turn on screen indicator");
        }

        let probname = CString::new("example").expect("CString::new failed");
        let lp = unsafe { CPXcreateprob(env, &mut status, probname.as_c_str().as_ptr()) };
        if lp.is_null() {
            panic!("Failed to create LP");
        }

        let zobj = unsafe {
            libc::malloc(NUMCOLS * std::mem::size_of::<libc::c_double>()) as *mut libc::c_double
        };
        let zrhs = unsafe {
            libc::malloc(NUMROWS * std::mem::size_of::<libc::c_double>()) as *mut libc::c_double
        };
        let zsense = unsafe {
            libc::malloc(NUMROWS * std::mem::size_of::<libc::c_char>()) as *mut libc::c_char
        };
        let zmathbeg = unsafe {
            libc::malloc(NUMCOLS * std::mem::size_of::<libc::c_int>()) as *mut libc::c_int
        };
        let zmathcnt = unsafe {
            libc::malloc(NUMCOLS * std::mem::size_of::<libc::c_int>()) as *mut libc::c_int
        };
        let zmathind =
            unsafe { libc::malloc(NUMNZ * std::mem::size_of::<libc::c_int>()) as *mut libc::c_int };
        let zmathval = unsafe {
            libc::malloc(NUMNZ * std::mem::size_of::<libc::c_double>()) as *mut libc::c_double
        };
        let zlb = unsafe {
            libc::malloc(NUMCOLS * std::mem::size_of::<libc::c_double>()) as *mut libc::c_double
        };
        let zub = unsafe {
            libc::malloc(NUMCOLS * std::mem::size_of::<libc::c_double>()) as *mut libc::c_double
        };
        let zctype = unsafe {
            libc::malloc(NUMCOLS * std::mem::size_of::<libc::c_char>()) as *mut libc::c_char
        };
        let obj = unsafe { std::slice::from_raw_parts_mut(zobj, NUMCOLS) };

        let rhs = unsafe { std::slice::from_raw_parts_mut(zrhs, NUMROWS) };
        let sense = unsafe { std::slice::from_raw_parts_mut(zsense, NUMROWS) };
        let mathbeg = unsafe { std::slice::from_raw_parts_mut(zmathbeg, NUMCOLS) };
        let mathcnt = unsafe { std::slice::from_raw_parts_mut(zmathcnt, NUMCOLS) };
        let mathind = unsafe { std::slice::from_raw_parts_mut(zmathind, NUMNZ) };
        let mathval = unsafe { std::slice::from_raw_parts_mut(zmathval, NUMNZ) };
        let lb = unsafe { std::slice::from_raw_parts_mut(zlb, NUMCOLS) };
        let ub = unsafe { std::slice::from_raw_parts_mut(zub, NUMCOLS) };
        let ctype = unsafe { std::slice::from_raw_parts_mut(zctype, NUMCOLS) };

        obj[0] = 1.0;
        obj[1] = 2.0;
        obj[2] = 3.0;
        obj[3] = 1.0;

        mathbeg[0] = 0;
        mathbeg[1] = 2;
        mathbeg[2] = 5;
        mathbeg[3] = 7;

        mathcnt[0] = 2;
        mathcnt[1] = 3;
        mathcnt[2] = 2;
        mathcnt[3] = 2;

        mathind[0] = 0;
        mathval[0] = -1.0;
        mathind[2] = 0;
        mathval[2] = 1.0;
        mathind[5] = 0;
        mathval[5] = 1.0;
        mathind[7] = 0;
        mathval[7] = 10.0;

        mathind[1] = 1;
        mathval[1] = 1.0;
        mathind[3] = 1;
        mathval[3] = -3.0;
        mathind[6] = 1;
        mathval[6] = 1.0;

        mathind[4] = 2;
        mathval[4] = 1.0;
        mathind[8] = 2;
        mathval[8] = -3.5;

        lb[0] = 0.0;
        lb[1] = 0.0;
        lb[2] = 0.0;
        lb[3] = 2.0;

        ub[0] = 40.0;
        ub[1] = 1e+20;
        ub[2] = 1e+20;
        ub[3] = 3.0;

        ctype[0] = 'C' as libc::c_char;
        ctype[1] = 'C' as libc::c_char;
        ctype[2] = 'C' as libc::c_char;
        ctype[3] = 'I' as libc::c_char;

        sense[0] = 'L' as libc::c_char;
        rhs[0] = 20.0;

        sense[1] = 'L' as libc::c_char;
        rhs[1] = 30.0;

        sense[2] = 'E' as libc::c_char;
        rhs[2] = 0.0;

        status = unsafe {
            CPXcopylp(
                env,
                lp,
                NUMCOLS as libc::c_int,
                NUMROWS as libc::c_int,
                CPX_MAX,
                zobj,
                zrhs,
                zsense,
                zmathbeg,
                zmathcnt,
                zmathind,
                zmathval,
                zlb,
                zub,
                std::ptr::null(),
            )
        };

        if status != 0 {
            panic!("Failed to copy problem data");
        }

        status = unsafe { CPXcopyctype(env, lp, zctype) };

        if status != 0 {
            panic!("Failed to copy ctype");
        }

        status = unsafe { CPXmipopt(env, lp) };

        if status != 0 {
            panic!("Failed to optimize MIP");
        }

        let solstat = unsafe { CPXgetstat(env, lp) };

        println!("Solution status = {}", solstat);

        let mut objval = 0.0;
        status = unsafe { CPXgetobjval(env, lp, &mut objval) };
        if status != 0 {
            panic!("No MIP objective value available");
        }

        assert_eq!(objval, 122.5);
    }
}
