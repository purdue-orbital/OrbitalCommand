FROM rustlang/rust:nightly AS builder

ARG TARGET_CRATE=ground

RUN apt update
RUN apt install -y libsoapysdr-dev libclang-dev clang

WORKDIR /usr/src/orbital

# Copy utility crates
COPY Cargo.lock ./
RUN printf "[workspace]\nmembers=[\"$TARGET_CRATE\"]" > Cargo.toml

RUN USER=root cargo new --bin $TARGET_CRATE

# Build external libraries
WORKDIR ./$TARGET_CRATE
COPY $TARGET_CRATE/Cargo.toml .
# Clear all path-based (local) packages
RUN sed --in-place '/path = "\.\./d' Cargo.toml
#RUN if [[ $TARGETARCH = "amd64" ]] ; then cargo build --target x86_64-unknown-linux-musl --release ; \
#    else cargo build --target aarch64-unknown-linux-musl --release ; fi
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/orbital/target \
    cargo build --release

# Copy and build internal libraries
WORKDIR /usr/src/orbital
COPY radio ./radio
RUN printf "[workspace]\nmembers=[\"radio\",\"$TARGET_CRATE\"]" > ./Cargo.toml

WORKDIR ./$TARGET_CRATE
COPY $TARGET_CRATE/Cargo.toml ./Cargo.toml

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/orbital/target \
    cargo build --release
RUN rm -r src

RUN --mount=type=cache,target=/usr/src/orbital/target \
    rm ../target/release/deps/$TARGET_CRATE*

# Build executable
# Copy actual source files
COPY $TARGET_CRATE/ .

RUN mkdir ../out
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/orbital/target \
    cargo build --release && mv ../target/release/$TARGET_CRATE ../out/app

FROM ubuntu AS runner

ARG APP=/usr/src

COPY --from=builder /usr/src/orbital/out/ ${APP}/tmp

WORKDIR ${APP}
RUN mv tmp/app .
RUN if [[ -d "tmp/dist" ]] ; then cp -r ./tmp/dist ./dist ; fi
RUN rm -r tmp

ENTRYPOINT ["./app"]