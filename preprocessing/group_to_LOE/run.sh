#!/bin/bash

# Run the script to group the data by LOE
set -e

raw_geojsons_directory="../../data/raw_geojsons"

find "$raw_geojsons_directory" -type f -name "LOE_*" | while read -r file; do
    # take off both the .geojson and the LOE_ prefix
    filename=$(basename -- "$file" | sed 's/LOE_//g' | sed 's/.geojson//g')
    echo "Processing ${filename}"

    cargo run --release ${filename}
done