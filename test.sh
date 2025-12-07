#!/usr/bin/env bash
# test.sh - run ./run.sh on every .c under testcases and compare to .out
# Usage:
#   ./test.sh            # run all tests
#   ./test.sh path/file.c  # run only that test
#   ./test.sh --update-missing   # create missing .out from current output
#   ./test.sh --accept-failures  # overwrite .out when test fails (use carefully)

set -u
BASE="testcases"
RUNNER="./run.sh"
TMPDIR=$(mktemp -d)
OK=0
NG=0
TOTAL=0
UPDATE_MISSING=0
ACCEPT_FAILS=0

# parse simple flags
while [[ $# -gt 0 && "${1:0:1}" = "-" ]]; do
  case "$1" in
    --update-missing) UPDATE_MISSING=1; shift ;;
    --accept-failures) ACCEPT_FAILS=1; shift ;;
    --) shift; break ;;
    *) echo "Unknown option: $1"; exit 1 ;;
  esac
done

# build list of targets
if [ $# -ge 1 ]; then
  TARGETS=()
  for arg in "$@"; do
    TARGETS+=("$arg")
  done
else
  # find all .c under BASE
  IFS=$'\n' read -r -d '' -a TARGETS < <(find "$BASE" -name "*.c" -print0 | xargs -0 -n1 echo | sort && printf '\0')
fi

if [ ${#TARGETS[@]} -eq 0 ]; then
  echo "No tests found."
  exit 1
fi

echo "Running ${#TARGETS[@]} tests..."

for SRC in "${TARGETS[@]}"; do
  TOTAL=$((TOTAL+1))
  NAME=$(basename "$SRC" .c)
  DIR=$(dirname "$SRC")
  EXPECT="$DIR/$NAME.out"
  IN="$DIR/$NAME.in"
  OUT="$TMPDIR/$NAME.result"
  ERR="$TMPDIR/$NAME.err"
  DIFF="$TMPDIR/$NAME.diff"

  echo "===== TEST: $SRC ====="

  if [ ! -x "$RUNNER" ]; then
    echo "ERROR: runner $RUNNER not found or not executable"
    exit 2
  fi

  # run ./run.sh; if there is a .in, feed it to stdin
  if [ -f "$IN" ]; then
    # redirect stderr to err file
    if ! "$RUNNER" "$SRC" < "$IN" > "$OUT" 2> "$ERR"; then
      RC=$?
      echo "  run.sh exited with $RC"
      echo "  stderr (first 40 lines):"
      sed -n '1,40p' "$ERR" || true
      NG=$((NG+1))
      echo
      continue
    fi
  else
    if ! "$RUNNER" "$SRC" > "$OUT" 2> "$ERR"; then
      RC=$?
      echo "  run.sh exited with $RC"
      echo "  stderr (first 40 lines):"
      sed -n '1,40p' "$ERR" || true
      NG=$((NG+1))
      echo
      continue
    fi
  fi

  # if expected missing
  if [ ! -f "$EXPECT" ]; then
    echo "  Missing expected file: $EXPECT"
    if [ "$UPDATE_MISSING" -eq 1 ]; then
      cp "$OUT" "$EXPECT"
      echo "  -> Generated expected (cp $OUT $EXPECT)"
      OK=$((OK+1))
    else
      echo "  (run with --update-missing to create it automatically)"
      NG=$((NG+1))
    fi
    echo
    continue
  fi

  # compare
  if diff -u --strip-trailing-cr "$EXPECT" "$OUT" > "$DIFF"; then
    echo "  PASS"
    OK=$((OK+1))
  else
    echo "  FAIL"
    echo "  diff:"
    sed -n '1,200p' "$DIFF" || true

    if [ "$ACCEPT_FAILS" -eq 1 ]; then
      cp "$OUT" "$EXPECT"
      echo "  -> Failed expected overwritten (accepted)."
      OK=$((OK+1))
    else
      NG=$((NG+1))
    fi
  fi
  echo
done

echo "========================"
echo "TOTAL: $TOTAL  PASS: $OK  FAIL: $NG"
echo "========================"

# cleanup
rm -rf "$TMPDIR"

# exit with non-zero if any failed
if [ "$NG" -ne 0 ]; then
  exit 3
fi