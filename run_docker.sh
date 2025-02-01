#!/bin/bash
set -e && cd "${0%/*}"
touch URL_shortens.kvm
docker build -t rust_webserver .
docker run --restart unless-stopped \
           -v $(pwd)/files:/app/files \
           -v $(pwd)/URL_shortens.kvm:/app/URL_shortens.kvm \
           -v $(pwd)/config.rs:/app/src/config.rs \
           -p 234:234 \
           -it rust_webserver