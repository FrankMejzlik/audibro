#!/bin/bash

BUILD_TYPE=debug
if [ "$1" = "release" ]; then
  BUILD_TYPE=$1
fi
echo "Running receiver as $BUILD_TYPE..."

mkdir -p env/receiver-carol/
cd env/receiver-carol/
../../target/$BUILD_TYPE/audibro receiver --tui "127.0.0.1:5001" alice
