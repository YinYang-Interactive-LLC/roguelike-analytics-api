#!/bin/sh -eu

cleanup() {
  docker stop roguelike-analytics-integration-test-redis
}

ROOT_DIR=`realpath $(dirname $0)/..`
PRODUCT_NAME="roguelike-analytics-ingest-server"
REDIS_PORT=${REDIS_PORT:-12345}
REDIS_PASSWORD="roguelike-analytics-integration-test-password"

cargo build --release

docker run -d --rm --name roguelike-analytics-integration-test-redis \
    -p $REDIS_PORT:6379 \
    redis:latest redis-server --requirepass "$REDIS_PASSWORD"

trap cleanup EXIT

# Wait a few secs
sleep 2

export REDIS_HOSTNAME="localhost"
export REDIS_PORT="${REDIS_PORT}"
export REDIS_PASSWORD="${REDIS_PASSWORD}"

"${ROOT_DIR}/tests/integration_test.py" "${ROOT_DIR}/target/release/${PRODUCT_NAME}" "http://localhost:${REDIS_PORT}"
