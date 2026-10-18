#!/usr/bin/env bash
# Turns the template into your app: replaces the names everywhere, renames the files
# that carry them, then removes itself and the README's template notes. Commit after.
# Usage: scripts/rename.sh "Display Name" binary app.id [crate-name] [owner/repo]
#   scripts/rename.sh "Junk Drawer" junk io.github.DisposableSoftware.JunkDrawer junk-drawer
# crate-name defaults to the display name in kebab case, owner/repo to the origin remote
# (or DisposableSoftware/<crate-name>).
set -euo pipefail
cd "$(dirname "$0")/.."

[ $# -ge 3 ] || { sed -n '4,7s/^# \{0,1\}//p' "$0" >&2; exit 2; }
name=$1 bin=$2 id=$3
crate=${4:-$(tr '[:upper:] ' '[:lower:]-' <<<"$name")}
repo=${5:-$(git remote get-url origin 2>/dev/null | sed -nE 's#.*github\.com[:/](.+/[^/]+?)(\.git)?$#\1#p')}
repo=${repo:-DisposableSoftware/$crate}
for v in "$bin" "$crate"; do
  [[ $v =~ ^[a-z][a-z0-9-]*$ ]] || { echo "'$v': use lowercase letters, digits and -" >&2; exit 2; }
done
[[ $id =~ ^[A-Za-z][A-Za-z0-9_]*(\.[A-Za-z][A-Za-z0-9_]*){2,}$ ]] || { echo "'$id' is not a reverse-DNS app ID" >&2; exit 2; }
git diff --quiet && git diff --cached --quiet || { echo "working tree is dirty" >&2; exit 1; }

old_id=io.github.DisposableSoftware.TemplateApp
upper=$(tr '[:lower:]-' '[:upper:]_' <<<"$bin")
esc() { sed 's/[&|\\]/\\&/g' <<<"$1"; }

git ls-files -z | grep -zv -e '^LICENSE$' -e '^scripts/rename\.sh$' | xargs -0 sed -i -E \
  -e "s|${old_id//./\\.}|$(esc "$id")|g" \
  -e "s|DisposableSoftware/template-app|$(esc "$repo")|g" \
  -e "s|template\\\\-app|$(esc "${crate//-/\\-}")|g" \
  -e "s|template-app|$crate|g" \
  -e "s|Template App|$(esc "$name")|g" \
  -e "s|\\bTAPP\\b|$upper|g" \
  -e "s|\\btapp\\b|$bin|g"

for f in data/"$old_id".*; do git mv "$f" "data/$id${f#data/"$old_id"}"; done
git mv data/tapp.1 "data/$bin.1"

# Everything between the template markers in the README is about the template itself.
sed -i '/<!-- template:begin -->/,/<!-- template:end -->/d' README.md
git rm -q scripts/rename.sh

echo "Renamed to $name ($bin, $id, crate $crate, github.com/$repo)."
echo "Next: nix develop -c cargo test, git config core.hooksPath .githooks, then commit."
