#!/bin/sh -eu

secret_key=${SECRET_KEY:-}

if [ -z "$secret_key" ]; then
  export SECRET_KEY=`echo $(openssl rand -hex 32) | sha256sum | tr -d ' -'`
  echo "SECRET_KEY was not defined. For this session '${SECRET_KEY}' will be used as a key."
fi

roguelike-analytics-api
