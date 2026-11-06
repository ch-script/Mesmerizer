#!/bin/sh
set -e

PREFIX="/usr/local/bin"
SNTD_BIN="sntd"
INIT_SRC="init/init.c"
INIT_OUT="init/init"

echo ":3 -------------------------------"
echo " Starting Mesmerizer Server Setup"
echo ":3 -------------------------------"

# Check dependencies
echo "[1/4] Checking required build tools..."
if ! command -v cc >/dev/null 2>&1; then
    echo "ERROR: C compiler (cc/clang) is not installed."
    exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo "ERROR: Rust and Cargo are not installed."
    exit 1
fi
echo "Build tools found."

# Compile C Init Script
echo "[2/4] Compiling FreeBSD initialization script (init.c)"
cc -O2 "$INIT_SRC" -o "$INIT_OUT"
echo "Successfully compiled $INIT_OUT."

# Compile Rust Workspace
echo "[3/4] Compiling Core Engine (sntd) via Cargo"
cargo build --release --bin core_engine
echo "Successfully compiled core_engine."

echo "[4/4] Installing binaries to $PREFIX..."
if [ "$(id -u)" -ne 0 ]; then
    echo "Notice: You are not root. The script will request sudo password to install the daemon."
    SUDO="sudo"
else
    SUDO=""
fi

# Copy the Rust binary as 'sntd' to the system LOCAL binaries folder
$SUDO cp target/release/core_engine "$PREFIX/$SNTD_BIN"
$SUDO chmod +x "$PREFIX/$SNTD_BIN"

echo "---------------------------"
echo " Installation Complete!"
echo " - Daemon installed at: $PREFIX/$SNTD_BIN"
echo " - Init script ready at: $INIT_OUT"
echo "---------------------------"
