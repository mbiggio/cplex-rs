# cplex-rs

Safe rust bindings for [CPLEX](https://www.ibm.com/products/ilog-cplex-optimization-studio/cplex-optimizer), based on the existing [rplex](https://github.com/emallson/rplex/tree/master) project. 
It adds a few features on top of `rplex`, such as:
* parameter namespaces consistent with the native C/C++ CPLEX API
* more structured error handling
* possibility to add variables and constraints in batch
* raw bindings generate at compile time parsing the actual cplex header

It also remove some features which are present `rplex`, especially the macros to define constraints and variables, since they present some issues.

It depends on the raw cplex bindings, generated through [bindgen](https://github.com/rust-lang/rust-bindgen) in the [cplex-rs-sys](./cplex-rs-sys/README.md) crate.
