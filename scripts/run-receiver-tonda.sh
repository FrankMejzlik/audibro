#!/bin/bash

BUILD_TYPE=debug
if [ "$1" = "release" ]; then
  BUILD_TYPE=$1
fi
echo "Running receiver as $BUILD_TYPE..."

cd env/receiver/
#../../target/$BUILD_TYPE/audibro receiver "195.113.19.166:6555" tonda
../../target/$BUILD_TYPE/audibro receiver "127.0.0.1:6555" tonda
