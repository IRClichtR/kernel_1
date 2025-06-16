#!/bin/bash

docker build env -t my-os-buildenv

docker run --rm -it \
    -v $PWD:/root/env my-os-buildenv \
    -w /root/env my-os-buildenv \
    -u $(id -u):$(id -g) \
    my-os-buildenv
