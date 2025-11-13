#!/bin/bash
# Wrapper script for running Verus verification
# Usage: ./scripts/verus.sh <file.rs> [additional args]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
VERUS_DIR="${PROJECT_ROOT}/tools/verus-bin"

# Check if Verus is installed
if [ ! -d "${VERUS_DIR}" ]; then
    echo "Verus not found. Installing..."
    "${SCRIPT_DIR}/install-verus.sh"
fi

# Find Verus binary
if [ -f "${VERUS_DIR}/verus" ]; then
    VERUS_BIN="${VERUS_DIR}/verus"
elif [ -f "${VERUS_DIR}/verus.exe" ]; then
    VERUS_BIN="${VERUS_DIR}/verus.exe"
else
    echo "Error: Verus binary not found in ${VERUS_DIR}"
    exit 1
fi

# Run Verus
exec "${VERUS_BIN}" "$@"
