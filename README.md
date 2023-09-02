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

All benchmarks are run on a Raspberry PI 4B running Raspbian aarch64.

### Radio

#### ASK
To Run:
```shell
cargo bench --features ask --manifest-path=radio/Cargo.toml
```
Result:
```
Mod 2048 Bytes:  [269.31 µs, 271.27 µs, 273.32 µs]
Demod 2048 Bytes:  [1.5399 ms 1.5415 ms 1.5433 ms]
```

#### FSK
To Run:
```shell
cargo bench --features fsk --manifest-path=radio/Cargo.toml
```
Result:
```
Mod 2048 Bytes:  [1.2463 ms, 1.2528 ms, 1.2596 ms]
Demod 2048 Bytes:  [5.0906 ms 5.1181 ms 5.1592 ms]
```

#### MFSK-8
To Run:
```shell
cargo bench --features mfsk --manifest-path=radio/Cargo.toml
```
Result:
```
Mod 2048 Bytes:  [7.0469 ms, 7.0577 ms, 7.0683 ms]
Demod 2048 Bytes:  [8.9959 ms, 8.9997 ms, 9.0036 ms]
```

#### BPSK
To Run:
```shell
cargo bench --features bpsk --manifest-path=radio/Cargo.toml
```
Result:
```
Mod 2048 Bytes:  [5.1202 ms 5.1264 ms 5.1342 ms]
Demod 2048 Bytes:  [7.1631 ms 7.1662 ms 7.1697 ms]
```

#### QPSK
To Run:
```shell
cargo bench --features qpsk --manifest-path=radio/Cargo.toml
```
Result:
```
Mod 2048 Bytes:  [6.8521 ms 7.3044 ms 7.7264 ms]
Demod 2048 Bytes:  [5.1894 ms 5.8040 ms 6.4660 ms]
```


## Testing

This section is for developers who wish to help contribute to this code! 
___ALL___ code needs to be tested before being committed.

### Radio

To test all radio components work properly, run the code seen below.

```shell
cargo test --manifest-path=radio/Cargo.toml
```

## Developer Introduction
![](./images/TCPModel.png) (Credit: https://stackoverflow.com/questions/31446777/difference-between-packets-and-frames)



