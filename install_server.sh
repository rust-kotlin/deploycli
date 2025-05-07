#!/bin/bash

# allow specifying different destination directory
DIR="${DIR:-"/root/deploycli"}"

# map different architecture variations to the available binaries
ARCH=$(uname -m)
case $ARCH in
    i386|i686) ARCH=x86 ;;
    aarch64*) ARCH=arm64 ;;
esac

# prepare the download URL
GITHUB_LATEST_VERSION=$(curl -L -s -H 'Accept: application/json' https://github.com/rust-kotlin/deploycli/releases/latest | sed -e 's/.*"tag_name":"\([^"]*\)".*/\1/')
GITHUB_FILE="deploycli-${ARCH}-unknown-linux-musl.tar.gz"
GITHUB_URL="https://github.com/rust-kotlin/deploycli/releases/download/${GITHUB_LATEST_VERSION}/${GITHUB_FILE}"

# install/update the local binary
curl -L -o deploycli.tar.gz $GITHUB_URL
tar xzvf deploycli.tar.gz deploycli
install -Dm 755 deploycli -t "$DIR"
rm deploycli deploycli.tar.gz
