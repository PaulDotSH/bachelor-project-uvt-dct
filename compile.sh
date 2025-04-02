#!/bin/bash

export DATABASE_URL=$(grep "DATABASE_URL" .env | awk -F '=' '{print $2}')
export RUSTFLAGS="-C target-cpu=native"
cargo build --profile production