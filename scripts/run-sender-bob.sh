#!/bin/bash

BUILD_TYPE=release
if [ "$1" = "debug" ]; then
  BUILD_TYPE=$1
fi
echo "Running sender as $BUILD_TYPE..."

cd env/sender-bob/
../../target/${BUILD_TYPE}/audibro --seed=42 --layers=3 --key-charges=3 sender "0.0.0.0:5556" bob
