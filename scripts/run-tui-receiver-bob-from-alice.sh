#!/bin/bash

BUILD_TYPE=debug
if [ "$1" = "release" ]; then
  BUILD_TYPE=$1
fi
echo "Running receiver as $BUILD_TYPE..."

cd env/receiver-bob/
../../target/$BUILD_TYPE/audibro receiver --tui "127.0.0.1:6555" alice