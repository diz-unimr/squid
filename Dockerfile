FROM rust:1.87.0-alpine3.22 AS build

RUN set -ex && \
    apk add --no-progress --no-cache \
        openssl-dev musl-dev openssl-libs-static

WORKDIR /app
COPY Cargo.toml Cargo.lock /app/
COPY ./src /app/src
COPY ./entity /app/entity
COPY ./migration /app/migration
COPY ./cert /app/cert
RUN cargo build --release

FROM alpine:3.22 AS run

RUN apk add --no-progress --no-cache tzdata

ENV UID=65532
ENV GID=65532
ENV USER=nonroot
ENV GROUP=nonroot

RUN addgroup -g $GID $GROUP && \
    adduser --shell /sbin/nologin --disabled-password \
    --no-create-home --uid $UID --ingroup $GROUP $USER

WORKDIR /app/
COPY --from=build /app/target/release/squid ./
COPY ./app.yaml ./
USER $USER
EXPOSE 3000

ENTRYPOINT ["/app/squid"]
