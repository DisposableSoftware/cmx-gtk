#! /usr/bin/env nix-shell
#! nix-shell -i bash -p jq
# Spec: LOOK-1.
# Runs scripts/manual/scale-check.sh in its own headless sway at scales 1, 1.25, 1.5
# and 2. Screenshots go to $OUT/<scale>/.
cd "$(dirname "$0")/../.." || exit 1
OUT=${OUT:-$(mktemp -d)}
status=0
for s in 1 1.25 1.5 2; do
  echo "== scale $s"
  mkdir -p "$OUT/$s"
  OUT=$OUT/$s HEADLESS_SCALE=$s scripts/manual/headless.sh scripts/manual/scale-check.sh || status=1
done
echo "screenshots in $OUT"
exit $status
