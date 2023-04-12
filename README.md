# OrbitalCommand
Purdue Orbital's Avionics and Ground Control Library, now with 100% more Rust!

### Building
Build the code:
```
./build.sh <ground|launch> [load]
```

First argument determines which station to build, second optional argument determines whether to load into Docker. 
If no second argument is given, a tarball file of the container is left in the root directory.
