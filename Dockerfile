FROM rust:slim-buster as builder
WORKDIR /code

ENV SQLX_OFFLINE=1
COPY . .
RUN cargo b --release \
    && strip target/release/asfraud

# 
FROM debian:buster-slim
WORKDIR /app
COPY --from=builder /code/target/release/asfraud .
ENTRYPOINT [ "./asfraud" ]
CMD []
