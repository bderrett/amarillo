FROM rust:1.40.0
WORKDIR /usr/src/app
COPY Cargo.lock .
COPY Cargo.toml .
RUN mkdir src\
    && echo "// dummy file" > src/lib.rs
RUN cargo build --release
COPY val_v2.pb .
COPY src src
COPY run.sh .
