# Dockerfile for OSS-Fuzz
# This file should be placed in google/oss-fuzz/projects/vc-status-list/
# Reference: https://google.github.io/oss-fuzz/getting-started/new-project-guide/rust-lang/

FROM gcr.io/oss-fuzz-base/base-builder-rust

# Install any system dependencies if needed
RUN apt-get update && apt-get install -y \
    make \
    autoconf \
    automake \
    libtool \
    pkg-config

# Clone the repository
RUN git clone --depth 1 https://github.com/shaileshp0110/token-status-list.git vc-status-list

WORKDIR $SRC/vc-status-list

