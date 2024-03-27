# CPLEX community edition image generation for testing

This folder contains a [docker file](./Dockerfile) to generate a docker image with an installation of the CPLEX Community Edition, used in the CI of this repo to run the tests.

To generate the docker image from this docker file, download a CPLEX CE installer from the [IBM ILOG CPLEX website](https://www.ibm.com/products/ilog-cplex-optimization-studio), and run:

```bash
CPLEX_INSTALLER="/path/to/cplex/ce/installer" docker build -t cplex-ce-solver -f Dockerfile .
```

To publish the image to a remote (private) repository, run:

```bash
docker tag cplex-ce-solver remote/repository/name:latest
docker push remote/repository/name:latest
```