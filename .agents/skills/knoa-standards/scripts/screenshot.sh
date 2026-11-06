#!/bin/sh
# Captures the Knoa Dev main window to the given PNG path. Works even when other windows
# cover it, because it captures by window ID instead of screen region.
set -eu
out=${1:?usage: screenshot.sh <out.png>}
id=$(swift -e '
import CoreGraphics
let windows = CGWindowListCopyWindowInfo(.optionOnScreenOnly, kCGNullWindowID) as! [[String: Any]]
let main = windows.first { $0["kCGWindowOwnerName"] as? String == "Knoa Dev" && $0["kCGWindowName"] as? String == "Knoa" && $0["kCGWindowLayer"] as? Int == 0 }
if let id = main?["kCGWindowNumber"] { print(id) }
')
[ -n "$id" ] || { echo "Knoa Dev window not found. Is make dev running?" >&2; exit 1; }
screencapture -x -o -l"$id" "$out"
