FROM rust:1.55.0-bullseye

RUN apt-get update
RUN apt-get install -y clang
RUN apt-get install -y libbtrfsutil-dev

WORKDIR /app
