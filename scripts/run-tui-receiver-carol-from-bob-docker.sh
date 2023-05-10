#!/bin/bash

mkdir -p env/receiver-carol/
cd env/receiver-carol/

../../target/release/audibro receiver --tui "$1:5001" alice
