#!/bin/sh
# Produce the graphs and library-backed abstractions for one target.
# usage: pipeline.sh <python> <rq checkout of the target> <graph dir> <rq 2.12 checkout> [rederive]
# The observer and probe abstractions always come from rq 2.12, so a mutant or
# another version is seen through the abstraction derived on 2.12. For another
# version, the observers of that version are computed as well (re-derivation).
set -eu
PY=$1
TARGET=$2
OUT=$3
BASE=$4
HERE=$(cd "$(dirname "$0")" && pwd)
for bound in 0 2; do
  "$PY" -I "$HERE/run.py" explore --rq "$TARGET" --out "$OUT" --bound "$bound"
done
"$PY" -I "$HERE/run.py" observe --rq "$BASE" --graphs "$OUT" --purity semantic
"$PY" -I "$HERE/run.py" probe --rq "$BASE" --graphs "$OUT" --depth 2
if [ "${5:-}" = rederive ]; then
  "$PY" -I "$HERE/run.py" observe --rq "$TARGET" --graphs "$OUT" --purity semantic
fi
