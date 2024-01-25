FROM rustlang/rust:nightly AS builder

ARG TARGET_CRATE=ground
ARG CARGO_BUILD=""

RUN apt-get update
#RUN apk add soapy-sdr-dev --repository=https://dl-cdn.alpinelinux.org/alpine/edge/testing
RUN apt-get install -y libsoapysdr-dev build-essential clang binaryen software-properties-common npm
RUN npm install -g sass

RUN rustup target add wasm32-unknown-unknown
RUN cargo install cargo-generate
RUN cargo install --features no_downloads --locked cargo-leptos

#RUN ldd /usr/lib/libSoapySDR.so && sleep 30

ENV RUSTFLAGS=-Ctarget-feature=-crt-static

WORKDIR /usr/src/orbital

# Copy and build internal libraries
WORKDIR /usr/src/orbital
COPY $TARGET_CRATE ./$TARGET_CRATE
COPY radio ./radio
COPY common ./common
RUN printf "[workspace]\nmembers=[\"radio\",\"common\",\"$TARGET_CRATE\"]\nresolver=\"2\"\n[profile.wasm-release]\ninherits = \"release\"\nopt-level = 'z'\nlto = true\ncodegen-units = 1\npanic = \"abort\"" > ./Cargo.toml


WORKDIR /usr/src/orbital/$TARGET_CRATE
# COPY $TARGET_CRATE/Cargo.toml ./Cargo.toml

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/orbital/target \
    cargo build --release

RUN mkdir /usr/src/orbital/out
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/orbital/target \
    cargo $CARGO_BUILD build --release && mv ../target/release/$TARGET_CRATE /usr/src/oribtal/out/app && if [ "$CARGO_BUILD" = "leptos" ]; then mv /usr/src/orbital/target/site /usr/src/orbital/out/; fi

EXPOSE 8080
# EXPOSE 27017
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT="site"

WORKDIR /usr/src/orbital/out
ENTRYPOINT ["./app"]
#ENTRYPOINT ["sleep", "infinity"]

#FROM alpine AS runner
#
#EXPOSE 80
#
#ARG APP=/usr/src
#
#RUN apk update
#RUN apk add soapy-sdr-dev --repository=https://dl-cdn.alpinelinux.org/alpine/edge/testing
#
#RUN apk add clang npm build-base
#
#COPY --from=builder /usr/src/orbital/out/ ${APP}/tmp
#
#WORKDIR ${APP}
#RUN mv tmp/app .
#RUN if [[ -d "tmp/dist" ]] ; then cp -r ./tmp/dist ./dist ; fi
#RUN rm -r tmp
#
#ENTRYPOINT ["./app"]
