#!/bin/sh -eu

docker build \
  --progress plain \
  -f deploy/Dockerfile \
  -t yy-ia/roguelike-analytics-ingest-server:$(cat VERSION) \
  .
