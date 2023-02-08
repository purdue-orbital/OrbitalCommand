FROM alpine AS builder
RUN apk update
RUN apk add git cmake make clang build-base llvm-static llvm-dev clang-static clang-dev

WORKDIR /usr/src/orbital
COPY . .
WORKDIR /usr/src/orbital/build
RUN cmake -D CMAKE_EXE_LINKER_FLAGS=\"-static\" ../

ARG TARGET_EXEC

RUN make ${TARGET_EXEC}
RUN mv ${TARGET_EXEC} program

FROM busybox

WORKDIR /usr/src
COPY --from=builder /usr/src/orbital/build/program ./program
ENTRYPOINT [ "./program" ]