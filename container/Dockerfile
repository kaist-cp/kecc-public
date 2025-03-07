FROM docker.io/library/rust:1.85.0-bullseye

# Extra dependencies
RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -yq --no-install-recommends \
        git \
        g++ \
        cmake \
        m4 \
        csmith \
        libcsmith-dev \
        creduce \
    && rm -rf /var/lib/apt/lists/*
