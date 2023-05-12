# OrbitalCommand
Purdue Orbital's Avionics and Ground Control Library, now with 100% more Rust!

### Building
Build the code:
```shell
./build.sh <ground|launch> [load]
```

First argument determines which station to build, second optional argument determines whether to load into Docker. 
If no second argument is given, a tarball file of the container is left in the root directory.

### Running the Ground Station
```shell
docker run --privileged -p 80:80 ground:latest
```

### Running the Launch Station
```shell
docker run --privileged launch:latest
```

## Benchmarks

All benchmarks are run on a Raspberry PI 4B running Raspbian.

### Radio

#### ASK
To Run:
```shell
cargo bench --features ask --manifest-path=radio/Cargo.toml
```
Result:
```
Mod 2048 Bytes:  [1.4402 ms, 1.5494 ms, 1.6884 ms]
Demod 2048 Bytes:  [2.8386 ms, 2.8494 ms, 2.8656 ms]
```

#### FSK
To Run:
```shell
cargo bench --features fsk --manifest-path=radio/Cargo.toml
```
Result:
```
Mod 2048 Bytes:  [3.4526 ms, 3.5275 ms, 3.26085 ms]
Demod 2048 Bytes:  [31.533 ms, 31.629 ms, 33.682 ms]
```

#### MFSK-8
To Run:
```shell
cargo bench --features mfsk --manifest-path=radio/Cargo.toml
```
Result:
```
Mod 2048 Bytes:  [8.0488 ms, 8.0799 ms, 8.1309 ms]
Demod 2048 Bytes:  [10.279 ms, 10.313 ms, 10.369 ms]
```

## Testing

This section is for developers who wish to help contribute to this code! 
___ALL___ code needs to be tested before being committed.

### Radio

To test all radio components work properly, run the code seen below.

```shell
cargo test --manifest-path=radio/Cargo.toml
```

