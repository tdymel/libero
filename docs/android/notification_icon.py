"""Writes the white status bar icon from logo.svg's bars. usage: notification_icon.py logo.svg out.xml"""
import re
import sys

svg_path, out_path = sys.argv[1], sys.argv[2]
svg = open(svg_path).read()
x0, y0, w, h = (float(v) for v in re.search(r'viewBox="([^"]+)"', svg).group(1).split())
# A square viewport, the wide logo centred in it.
pad = (w - h) / 2
paths = []
for m in re.finditer(r'<rect x="([\d.]+)" y="([\d.]+)" width="([\d.]+)" height="([\d.]+)"', svg):
    x, y, _, rh = (float(v) for v in m.groups())
    # The bars fill their 5.6 pitch: 3.8 wide bars blur to grey stripes at 24dp.
    paths.append(f"M{x - x0:.1f},{y - y0 + pad:.1f}h5.6v{rh:.1f}h-5.6z")
with open(out_path, "w") as f:
    f.write('<?xml version="1.0" encoding="utf-8"?>\n')
    f.write("<!-- The status bar icon libero's notifications look up by name: logo.svg in white. -->\n")
    f.write('<vector xmlns:android="http://schemas.android.com/apk/res/android"\n')
    f.write('    android:width="24dp"\n    android:height="24dp"\n')
    f.write(f'    android:viewportWidth="{w}"\n    android:viewportHeight="{w}">\n')
    f.write('    <path\n        android:fillColor="#FFFFFFFF"\n')
    f.write(f'        android:pathData="{"".join(paths)}" />\n')
    f.write("</vector>\n")
