# cplex-rs ![CI Pipeline](https://github.com/mbiggio/cplex-rs/actions/workflows/ci.yml/badge.svg)

Safe rust bindings for [CPLEX](https://www.ibm.com/products/ilog-cplex-optimization-studio/cplex-optimizer), based on the existing [rplex](https://github.com/emallson/rplex/tree/master) project. 
It adds a few features on top of `rplex`, such as:
* parameter namespaces consistent with the native C/C++ CPLEX API
* more structured error handling
* possibility to add variables and constraints in batch
* raw bindings generate at compile time parsing the actual cplex header

It also remove some features which are present `rplex`, especially the macros to define constraints and variables, since they present some issues.

It depends on the raw cplex bindings, generated through [bindgen](https://github.com/rust-lang/rust-bindgen) in the [cplex-rs-sys](./cplex-rs-sys/README.md) crate.

To build this repo, a valid CPLEX installation must be present on the system. If the CPLEX installation path is not the standard one, you will need to pass the `CPLEX_PATH` environment variable to `cargo` with the cplex installation path of choice, e.g.:

```bash
CPLEX_PATH=/path/to/cplex/installation cargo build
```

## Testing
Tests in the CI are run using a personal copy of the CPLEX community edition, deployed in a docker image a private docker repository.

If you want to generate an image with your own CPLEX installation to test this repo, you can find instructions in the [./utils](./utils/README.md) folder on how to do so.