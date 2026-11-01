#!/usr/bin/env bash
set -euo pipefail

CC="${CC:-x86_64-w64-mingw32-gcc}"
CFLAGS="-O2 -Wall -Wextra -shared -s"

echo "[1/5] Compiling stardesk-hook.dll..."
$CC $CFLAGS -o stardesk-hook.dll stardesk-hook.c -lws2_32

echo "[2/5] Compiling wevtapi.dll..."
$CC $CFLAGS -o wevtapi.dll wevtapi.c wevtapi.def -L. -lstardesk-hook

echo "[3/5] Compiling wtsapi32.dll..."
$CC $CFLAGS -o wtsapi32.dll wtsapi32.c wtsapi32.def

echo "[4/5] Compiling launch-proxy.exe..."
$CC -O2 -Wall -s -o launch-proxy.exe launch-proxy.c -lws2_32

echo "[5/5] Compiling terminal-proxy.exe..."
$CC -O2 -Wall -s -o terminal-proxy.exe terminal-proxy.c -lws2_32

echo "All Wine hooks compiled successfully."
