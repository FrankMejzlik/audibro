#!/bin/bash

BUILD_TYPE=release
if [ "$1" = "debug" ]; then
  BUILD_TYPE=$1
fi
echo "Running receiver as $BUILD_TYPE..."

mkdir -p env/receiver-bob/
cd env/receiver-bob/
../../target/$BUILD_TYPE/audibro receiver --tui --distribute="127.0.0.1:5001" "127.0.0.1:5000" alice
