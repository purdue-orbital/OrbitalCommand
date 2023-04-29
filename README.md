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

To preform benchmarks for the radio, run this command

```shell
cargo bench --features bench --manifest-path=radio/Cargo.toml
```

#### ASK
```
Mod 2048 Bytes:  [3.7731 ms, 3.7809 ms, 3.7924 ms]
Demod 2048 Bytes:  [3.6864 ms, 3.6943 ms, 3.7079 ms]
```

## Testing

This section is for developers who wish to help contribute to this code! 
___ALL___ code needs to be tested before being committed.

### Radio

To test all radio components work properly, run the code seen below.

```shell
cargo test --manifest-path=radio/Cargo.toml
```

