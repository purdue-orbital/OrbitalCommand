#!/bin/sh

case $1 in
ground)
    docker buildx build --platform=linux/amd64,linux/arm64,linux/arm/v7 -t ground_station:latest --build-arg TARGET_EXEC=ground_station .
    if [ "$2" = "load" ]; then
        docker buildx build --load -t ground_station:latest --build-arg TARGET_EXEC=ground_station .
    elif [ "$2" = "savepi" ]; then
        mkdir -p build
        docker buildx build --output type=tar,dest=build/docker.tar --platform=linux/arm/v7 -t ground_station:latest --build-arg TARGET_EXEC=ground_station .
    fi
    ;;
launch)
    docker buildx build --platform=linux/amd64,linux/arm64,linux/arm/v7 -t launch_station:latest --build-arg TARGET_EXEC=launch_station .
    if [ "$2" = "load" ]; then
        docker buildx build --load -t launch_station:latest --build-arg TARGET_EXEC=launch_station .
    elif [ "$2" = "savepi" ]; then
        mkdir -p build
        docker buildx build --output type=tar,dest=build/docker.tar --platform=linux/arm/v7 -t launch_station:latest --build-arg TARGET_EXEC=launch_station .
    fi
    ;;
*)
    >&2 echo "Invalid first argument! Either use 'ground' or 'launch'"
    exit 1
esac