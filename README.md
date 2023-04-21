# OrbitalCommand
Purdue Orbital's Avionics and Ground Control Library, now with 100% more Rust!

### Building
Build the code:
```shell
./build.sh <ground|launch|radio> [load]
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

### Running Radio
```shell
docker run --privileged radio:latest
```

## Benchmarks

All benchmarks are run on a raspberry pi 4B

### Radio

#### ASK
```
Mod 2048 Bytes:  [8.6000 ms, 8.6052 ms, 8.6120 ms]
Demod 2048 Bytes:  [8.6120 ms, 8.6270 ms, 8.6432 ms]
```