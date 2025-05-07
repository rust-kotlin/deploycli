#!/bin/bash

# allow specifying different destination directory
DIR="${DIR:-"/usr/local/bin"}"

# map different architecture variations to the available binaries
ARCH=$(uname -m)
case $ARCH in
    i386|i686) ARCH=x86 ;;
    aarch64*) ARCH=arm64 ;;
esac

# prepare the download URL
GITHUB_LATEST_VERSION=$(curl -L -s -H 'Accept: application/json' https://github.com/rust-kotlin/deploycli/releases/latest | sed -e 's/.*"tag_name":"\([^"]*\)".*/\1/')
GITHUB_FILE="client-${ARCH}-unknown-linux-musl.tar.gz"
GITHUB_URL="https://github.com/rust-kotlin/deploycli/releases/download/${GITHUB_LATEST_VERSION}/${GITHUB_FILE}"

# install/update the local binary
curl -L -o client.tar.gz $GITHUB_URL
tar xzvf client.tar.gz client
install -Dm 755 client -t "$DIR"
rm client client.tar.gz
