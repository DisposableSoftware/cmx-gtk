#! /usr/bin/env nix-shell
#! nix-shell -i bash -p imagemagick zip dejavu_fonts fontconfig
# Draws tests/fixtures/demo.cbz: three original pages in the icon's palette, for the
# README and AppStream screenshot. Re-run to regenerate; the output is committed.
set -euo pipefail
cd "$(dirname "$0")/.."
font=$(fc-match -f '%{file}' 'DejaVu Sans:bold')
work=$(mktemp -d); trap 'rm -rf "$work"' EXIT
paper='#f4efe3' ink='#1f2433' accent='#ef6a43' sky='#7fb2d6' grass='#8cbf7a' dusk='#4a4e7a'

panel() { # X0 Y0 X1 Y1 FILL
  echo "fill '$5' stroke '$ink' stroke-width 10 rectangle $1,$2 $3,$4"
}
bubble() { # X Y W H TEXT
  echo "fill white stroke '$ink' stroke-width 6"
  echo "roundrectangle $1,$2 $(($1 + $3)),$(($2 + $4)) 40,40"
  echo "polygon $(($1 + 40)),$(($2 + $4 - 4)) $(($1 + 75)),$(($2 + $4 - 4)) $(($1 + 30)),$(($2 + $4 + 50))"
  echo "stroke none fill '$ink' font '$font' font-size 36"
  echo "text $(($1 + 30)),$(($2 + $4 / 2 + 13)) '$5'"
}
stars() { # X,Y...
  echo "fill '$paper' stroke none"
  local x y
  for p in "$@"; do IFS=, read -r x y <<<"$p"; echo "circle $x,$y $((x + 7)),$y"; done
}
page() { # NAME, MVG on stdin
  cat >"$work/$1.mvg"
  magick -size 1000x1500 "xc:$paper" -draw "@$work/$1.mvg" -depth 8 "$work/$1"
}

{
  panel 60 60 940 520 "$sky"
  echo "fill '$grass' stroke none rectangle 65,400 935,515"
  echo "fill '$accent' stroke '$ink' stroke-width 8 circle 760,200 760,260"
  bubble 110 110 360 110 'Another page?'
  panel 60 560 480 1440 "$dusk"
  stars 150,700 300,820 390,640 200,1100 400,1300
  panel 520 560 940 1440 "$accent"
  bubble 560 620 240 110 'Just one.'
} | page 01.png

{
  panel 60 60 940 760 "$dusk"
  stars 200,200 520,140 820,300 680,600 300,560
  bubble 120 480 420 110 'It is late, though.'
  panel 60 800 480 1440 "$grass"
  panel 520 800 940 1440 "$sky"
  bubble 560 860 180 110 'Space.'
} | page 02.png

{
  panel 60 60 940 1440 "$accent"
  echo "fill '$paper' stroke '$ink' stroke-width 10 roundrectangle 260,420 740,1080 30,30"
  echo "fill '$sky' stroke '$ink' stroke-width 8 rectangle 310,470 690,700"
  echo "fill '$ink' stroke none font '$font' font-size 110 text 395,940 'cmx'"
  bubble 140 160 420 110 'The end. Press q.'
} | page 03.png

cat >"$work/ComicInfo.xml" <<'XML'
<?xml version="1.0" encoding="utf-8"?>
<ComicInfo>
  <Series>Night Reading</Series>
  <Number>1</Number>
  <Title>Just One More Page</Title>
  <Summary>A reader promises to stop after one more page. A demo comic drawn for cmx's screenshots.</Summary>
  <Writer>cmx</Writer>
  <Penciller>ImageMagick</Penciller>
  <Publisher>DisposableSoftware</Publisher>
  <Year>2026</Year>
  <Month>10</Month>
  <Genre>Slice of Life</Genre>
  <LanguageISO>en</LanguageISO>
</ComicInfo>
XML
rm -f tests/fixtures/demo.cbz
(cd "$work" && zip -q -X "$OLDPWD/tests/fixtures/demo.cbz" ComicInfo.xml 01.png 02.png 03.png)
echo "wrote tests/fixtures/demo.cbz"
