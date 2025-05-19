if [ -z "$TARGETARCH" ]; then
  :
else
  case "${TARGETARCH}" in
    "amd64")
      LINKER_NAME="x86_64-linux-gnu-gcc"
      LINKER_PACKAGE="gcc-x86-64-linux-gnu"
      BUILD_TARGET="x86_64-unknown-linux-gnu"
      TOOLCHAIN="nightly-x86_64-unknown-linux-gnu" ;;
    "arm64")
      LINKER_NAME="aarch64-linux-gnu-gcc"
      LINKER_PACKAGE="gcc-aarch64-linux-gnu"
      BUILD_TARGET="aarch64-unknown-linux-gnu"
      TOOLCHAIN="nightly-aarch64-unknown-linux-gnu" ;;
  esac
fi

tools() {
  if [ -z "$TARGETARCH" ]; then
    :
  else
    apt-get install -y "${LINKER_PACKAGE}"
    rustup target add "${BUILD_TARGET}"
    rustup component add rust-src --toolchain "${TOOLCHAIN}"
  fi
}

deps() {
  if [ -z "$TARGETARCH" ]; then
    cargo build -j 10 --release
  else
    cargo build -j 10 --release
  fi
}

apps() {
  if [ -z "$TARGETARCH" ]; then
    cargo build -j 10 --release
  else
    cargo build -j 10 --release
  fi
}

if [ -z "$TARGETARCH" ]; then
  :
else
  export RUSTFLAGS="-C linker=${LINKER_NAME}"
  export PKG_CONFIG_ALLOW_CROSS="1"
  export PKG_CONFIG_PATH="/usr/lib/pkgconfig:/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/lib/x86_64-linux-gnu/pkgconfig"
fi

"$@"