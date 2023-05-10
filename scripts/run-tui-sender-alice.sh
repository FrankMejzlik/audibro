#!/bin/bash

BUILD_TYPE=debug
if [ "$1" = "release" ]; then
  BUILD_TYPE=$1
fi
echo "Running TUI sender as $BUILD_TYPE..."

mkdir -p env/receiver-alice/
cd env/sender-alice/
../../target/${BUILD_TYPE}/audibro --seed=42 --tui sender "0.0.0.0:5000" alice
