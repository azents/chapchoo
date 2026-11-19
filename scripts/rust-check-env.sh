#!/usr/bin/env bash
# Sourced by quality checks. Keep explicit Cargo profile overrides.
export CARGO_PROFILE_DEV_DEBUG="${CARGO_PROFILE_DEV_DEBUG-0}"
export CARGO_PROFILE_TEST_DEBUG="${CARGO_PROFILE_TEST_DEBUG-0}"
