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

## Benchmarks

All benchmarks are run on a Raspberry PI 4B running Raspbian.

### Radio

#### ASK
```
Mod 2048 Bytes:  [3.9191 ms, 3.9346 ms, 3.9571 ms]
Demod 2048 Bytes:  [3.9783 ms, 3.9877 ms, 4.0022 ms]
```