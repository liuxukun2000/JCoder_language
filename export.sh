#!/bin/sh

if [ $# -ne 2 ]
then
    echo "Usage: $0 <image> <dir>" 1>&2
    exit 1
fi

IMAGE="$1"
DIR="$2"

mkdir -p "$DIR"

CONTAINER=$(docker create "$IMAGE")
docker export "$CONTAINER" | tar -C "$DIR" -xvf -

docker container rm "$CONTAINER"