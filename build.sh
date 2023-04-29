#!/bin/bash

if [ -z "$1" ]; then
  echo "Error: argument 'ground' or 'launch' required"
  exit 1
elif [ "$1" != "ground" ] && [ "$1" != "launch" ] && [ "$1" != "radio" ]; then
  echo "Error: first argument must be 'ground', 'launch', or 'radio'"
  exit 1
fi

if [ -n "$2" ]; then
  if [ "$2" != "load" ]; then
    echo "Error: second argument must be 'load'"
    exit 1
  fi
fi

if [ "$1" = "ground" ]; then
  echo "Ground Station Build"
  # Run ground command here
  docker buildx build --build-arg MAPBOX_TOKEN="${MAPBOX_TOKEN}" --output type=docker,dest=./image.tar -t "ground:latest" .
elif [ "$1" = "launch" ]; then
  echo "Launch Station Build"
  # Run launch command here
  docker buildx build --build-arg TARGET_CRATE=launch --output type=docker,dest=./image.tar -t "launch:latest" .
elif [ "$1" = "radio" ]; then
  echo "Radio Build"
  # Run launch command here
  docker buildx build --build-arg TARGET_CRATE=radio --output type=docker,dest=./image.tar -t "radio:latest" .
fi

if [ "$2" = "load" ]; then
  echo "Loading Image..."
  # Run load command here
  docker load --input image.tar
fi
