#!/bin/bash

cargo build -q --release && \
    ./target/release/chorus ./sample/sample.config.toml
