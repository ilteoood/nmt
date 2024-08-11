FROM clux/muslrust:stable AS builder
COPY . .
RUN cargo build --release
RUN mv ./target/*-unknown-linux-musl/release/cli ./cli
RUN chmod +x ./cli

FROM scratch
COPY --from=builder /volume/cli ./cli