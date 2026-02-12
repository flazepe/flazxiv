FROM rust:1-alpine AS builder
WORKDIR /flazxiv
COPY . .
RUN apk update && apk upgrade
RUN apk add musl-dev openssl-dev pkgconf
RUN RUSTFLAGS="-Ctarget-feature=-crt-static" cargo install --path .
CMD ["flazxiv"]
