FROM ubuntu:latest

ARG CPLEX_INSTALLER=cos_installer_preview-22.1.1.0.R0-M08SWML-linux-x86-64.bin

COPY ${CPLEX_INSTALLER} /cplex/cplex_studio.linux-x86-64.bin
COPY response.properties /cplex/response.properties

RUN chmod u+x /cplex/cplex_studio.linux-x86-64.bin
RUN /cplex/cplex_studio.linux-x86-64.bin -f /cplex/response.properties
RUN rm -rf /cplex

RUN apt update && apt -y dist-upgrade && apt install -y curl build-essential llvm-dev libclang-dev clang