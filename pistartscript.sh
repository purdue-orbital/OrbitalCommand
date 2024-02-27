#!/bin/bash

mkdir -p ~/launchlogs
docker run -v ~/launchlogs:/var/log/orbital --privileged launch:latest
