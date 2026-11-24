#!/usr/bin/env bash
# Build one donor's evaluation DLL: the Stockfish tag, this directory's
# patch, MSYS2's MinGW g++, statically linked so it needs no MinGW DLLs.
# Usage: build_dll.sh <sf11|sf16> <stockfish clone> <out dir>
set -euo pipefail
V=$1; SF=$2; OUT=$(mkdir -p "$3" && cd "$3" && pwd)
HERE=$(cd "$(dirname "$0")" && pwd)
export PATH=/c/msys64/mingw64/bin:$PATH
case $V in
  sf11) TAG=sf_11; STD=c++11; EXTRA="" ;;
  # The network is never evaluated: not embedded, its SIMD paths only compiled.
  sf16) TAG=sf_16; STD=c++17; EXTRA="-DNNUE_EMBEDDING_OFF -DUSE_SSE41 -DUSE_SSSE3 -DUSE_SSE2 -DUSE_AVX2 -mavx2 -msse4.1 -mssse3" ;;
  *) echo "unknown donor $V" >&2; exit 2 ;;
esac
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
# The repository's own bytes (LF), which the patch is made against.
git -C "$SF" -c core.autocrlf=false archive "$TAG" src | tar -x -C "$WORK"
(cd "$WORK" && git apply "$HERE/$V-bridge.patch")
cd "$WORK/src"
FILES=$(ls *.cpp syzygy/*.cpp $( [ -d nnue ] && echo nnue/*.cpp nnue/features/*.cpp ) | grep -v '^main.cpp$')
g++ --version | head -1
g++ -std=$STD -O3 -DNDEBUG -DIS_64BIT -DUSE_POPCNT -DUSE_PEXT -msse -msse3 -mpopcnt -mbmi2 $EXTRA \
    -fno-exceptions -flto -shared -static -static-libgcc -static-libstdc++ \
    -o "$OUT/${V}_eval.dll" $FILES
echo "built $OUT/${V}_eval.dll from $TAG $(git -C "$SF" rev-parse "$TAG^{commit}")"
