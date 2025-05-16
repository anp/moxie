#!/usr/bin/env bash

bazel run @rules_rust//tools/rust_analyzer:discover_bazel_rust_project -- \
    --bazel_startup_option=--output_base=~/ide_bazel \
    --bazel_arg=--watchfs \
    2>/dev/null
