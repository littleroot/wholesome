FROM rust:1.47.0 AS builder

ARG SERVICE_NAME=wholesome

RUN USER=root cargo new --bin /usr/src/${SERVICE_NAME}

WORKDIR /usr/src/${SERVICE_NAME}

COPY Cargo.toml Cargo.toml

# cache deps
RUN cargo build --release && rm -rf src

ADD . ./

RUN rm ./target/release/deps/${SERVICE_NAME}* && cargo build --release

FROM gcr.io/distroless/cc-debian10

ARG SERVICE_NAME=wholesome

COPY --from=builder /usr/src/${SERVICE_NAME}/target/release/${SERVICE_NAME} \
    /usr/local/bin/${SERVICE_NAME}

CMD ["wholesome"]
