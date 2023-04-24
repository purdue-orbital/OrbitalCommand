FROM rustlang/rust:nightly AS builder

ARG TARGET_CRATE=ground

RUN apt-get update
#RUN apk add soapy-sdr-dev --repository=https://dl-cdn.alpinelinux.org/alpine/edge/testing
RUN apt-get install -y libsoapysdr-dev build-essential clang
RUN apt-get update && apt-get install -y \
    software-properties-common \
    npm
RUN npm install npm@latest -g && \
    npm install n -g && \
    n latest


#RUN ldd /usr/lib/libSoapySDR.so && sleep 30

ENV RUSTFLAGS=-Ctarget-feature=-crt-static

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

# Client shit
RUN mkdir client
COPY $TARGET_CRATE/client/*.* ./client/
WORKDIR ./client
RUN if [ -e "package.json" ] ; then npm install ; fi

# Copy and build internal libraries
WORKDIR /usr/src/orbital
COPY radio ./radio
COPY common ./common
RUN printf "[workspace]\nmembers=[\"radio\",\"common\",\"$TARGET_CRATE\"]" > ./Cargo.toml

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

WORKDIR client
RUN if [ -e "package.json" ] ; then npm run build ; fi
RUN if [ -e "package.json" ] ; then mkdir -p ../../out/dist ; fi
RUN if [ -e "package.json" ] ; then mv dist/* ../../out/dist ; fi

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