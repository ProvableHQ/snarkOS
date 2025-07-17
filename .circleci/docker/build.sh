#! /bin/bash

docker-buildx build .  --platform linux/amd64 -t cskama/snarkos-ci:4.0.0
