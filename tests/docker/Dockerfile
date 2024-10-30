
FROM debian:bookworm-slim as mimalloc-debian

RUN apt update && apt upgrade -y && apt install -y gcc g++ make cmake git
RUN git clone https://github.com/microsoft/mimalloc && \
  cd mimalloc && \
  git checkout 43ce4bd7fd34bcc730c1c7471c99995597415488 && \
  mkdir -p out/secure && \
  cd out/secure && \
  cmake -DMI_SECURE=ON ../.. && \
  make && \
  cp ./libmimalloc-secure.so ../../../libmimalloc.so

FROM rust:1.80-slim-bookworm as builder

COPY --from=mimalloc-debian libmimalloc.so /usr/lib
RUN echo "/usr/lib/libmimalloc.so" >> /etc/ld.so.preload

RUN apt update && apt upgrade -y && apt autoremove -y && apt clean

# Add dev dependencies
RUN apt install -y pkg-config clang

# Dependencies for the Wikiblocks node
RUN apt install -y make protobuf-compiler

# Add the wasm toolchain
RUN rustup target add wasm32-unknown-unknown

# Add files for build
ADD src /wikiblocks/src
ADD tests /wikiblocks/tests
ADD Cargo.toml /wikiblocks
ADD Cargo.lock /wikiblocks

WORKDIR /wikiblocks

# Mount the caches and build
RUN --mount=type=cache,target=/root/.cargo \
  --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  --mount=type=cache,target=/wikiblocks/target \
  mkdir /wikiblocks/bin && \
  cargo build --release --features "" -p wikiblocks-node && \
  mv /wikiblocks/target/release/wikiblocks-node /wikiblocks/bin

FROM debian:bookworm-slim as image

COPY --from=mimalloc-debian libmimalloc.so /usr/lib
RUN echo "/usr/lib/libmimalloc.so" >> /etc/ld.so.preload

RUN apt update && apt upgrade -y && apt autoremove -y && apt clean

RUN useradd --system --user-group --create-home --shell /sbin/nologin wikiblocks

# Make the /volume directory and transfer it to the user
RUN mkdir /volume && chown wikiblocks:wikiblocks /volume

# Switch to a non-root user
USER wikiblocks

WORKDIR /home/wikiblocks

# Copy the Wikiblocks binary and relevant license
COPY --from=builder --chown=wikiblocks /wikiblocks/bin/wikiblocks-node /bin/

# Run the Wikiblocks node
EXPOSE 30333 9944

ADD /tests/docker/run.sh /
CMD KEY=${KEY:="6f5aec7d703870c6dd4d9d96f110dc44758c5dcdfa5d9a5a8e54f91499adbb0d"}  "/run.sh"
