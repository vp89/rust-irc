## Builder

FROM rust:1.57.0 AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

ENV USER=rustirc
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

WORKDIR /rust-irc

COPY ./ .

RUN cargo build --target x86_64-unknown-linux-musl --release

## Final image

FROM scratch

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /rust-irc

COPY --from=builder /rust-irc/target/x86_64-unknown-linux-musl/release/rust-irc ./
COPY --from=builder /rust-irc/Settings.toml ./

USER rustirc:rustirc

CMD ["/rust-irc/rust-irc"]
