#!/bin/bash

BUILD_TYPE=release
if [ "$1" = "debug" ]; then
  BUILD_TYPE=$1
fi
echo "Running receiver as $BUILD_TYPE..."

cd env/receiver-bob/
../../target/$BUILD_TYPE/audibro receiver --distribute="127.0.0.1:5556" "127.0.0.1:5555" alice
