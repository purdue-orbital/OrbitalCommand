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

All benchmarks are run on a raspberry pi 4B running raspbian.

### Radio

#### ASK
```
Mod 2048 Bytes:  [18.673 ms, 18.696 ms, 18.728 ms]
Demod 2048 Bytes:  [18.651 ms, 18.673 ms, 18.701 ms]
```