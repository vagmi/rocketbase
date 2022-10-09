# -*- mode: dockerfile -*-
#
# An example Dockerfile showing how to build a Rust executable using this
# image, and deploy it with a tiny Alpine Linux container.

# You can override this `--build-arg BASE_IMAGE=...` to use different
# version of Rust or OpenSSL.
ARG BASE_IMAGE=ekidd/rust-musl-builder:latest

# Our first FROM statement declares the build environment.
FROM ${BASE_IMAGE} AS builder

# Add our source code.
ADD --chown=rust:rust . ./

# Build our application.
RUN cargo build --release

# Now, we need to build our _real_ Docker container, copying in `using-sqlx`.
FROM alpine:latest

EXPOSE 3000
# RUN apk --no-cache add ca-certificates
COPY settings settings/
COPY --from=builder \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/rocketbase \
    /usr/local/bin/
CMD /usr/local/bin/rocketbase
