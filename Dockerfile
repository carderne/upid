ARG PG_MAJOR=16

# ##################################
# builder image
# ##################################
FROM postgres:${PG_MAJOR} AS builder

RUN apt-get -y update && \
  apt-get install -y --no-install-recommends \
  pkg-config \
  cmake \
  ca-certificates \
  git \
  build-essential \
  libpq-dev \
  postgresql-server-dev-${PG_MAJOR} \
  curl \
  libreadline6-dev \
  zlib1g-dev

ENV HOME=/home/builder
ENV PATH=/home/builder/.cargo/bin:$PATH
WORKDIR /home/builder

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path --profile minimal --default-toolchain 1.79.0
RUN cargo install cargo-pgrx --version 0.11.4 --locked
RUN cargo pgrx init --pg${PG_MAJOR} $(which pg_config)

COPY .cargo .cargo
COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml
COPY upid_pg upid_pg
COPY upid_rs upid_rs
RUN cargo pgrx install --package upid_pg

# ##################################
# runner image
# ##################################
FROM postgres:${PG_MAJOR} AS runner

COPY --from=builder /usr/share/postgresql/${PG_MAJOR}/extension /usr/share/postgresql/${PG_MAJOR}/extension
COPY --from=builder /usr/lib/postgresql/${PG_MAJOR}/lib /usr/lib/postgresql/${PG_MAJOR}/lib

RUN chown -R postgres:postgres /usr/share/postgresql/${PG_MAJOR}/extension
RUN chown -R postgres:postgres /usr/lib/postgresql/${PG_MAJOR}/lib

USER postgres
ENV USER=postgres
