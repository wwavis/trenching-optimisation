#!/bin/bash

set -e

mkdir -p ../../data/raw_geojsons

# Enter filename
read -p "Enter file path to real_sitedata folder, e.g. (../../../Trenchscrits/real_sitedata):" shape_directory

# shape_directory="../../../Trenchscrits/real_sitedata"

find "$shape_directory" -type f -name "*.shp" | while read -r file; do
    filename=$(basename "$file" .shp)
    cargo run --release ${file} ../../data/raw_geojsons/${filename}.geojson
done