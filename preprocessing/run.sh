#!/bin/bash

shape_directory="../../Trenchscrits/real_sitedata"

find "$shape_directory" -type f -name "*.shp" | while read -r file; do
    filename=$(basename "$file" .shp)
    cargo run --release ${file} ../data/${filename}.geojson
done