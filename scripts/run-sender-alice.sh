#!/bin/bash

BUILD_TYPE=release
if [ "$1" = "debug" ]; then
  BUILD_TYPE=$1
fi
echo "Running sender as $BUILD_TYPE..."

cd env/sender-alice/
../../target/${BUILD_TYPE}/audibro --seed=40 --key-charges=3 --max-piece-size=10485760 sender "0.0.0.0:5555" alice
