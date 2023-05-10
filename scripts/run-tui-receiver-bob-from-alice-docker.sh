#!/bin/bash


mkdir -p env/receiver-bob/
cd env/receiver-bob/

../../target/release/audibro receiver --tui --distribute="127.0.0.1:5001" "$1:5000" alice
