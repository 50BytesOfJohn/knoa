# dmgbuild settings for Knoa.dmg, run by package.sh. It writes the window layout into the
# image directly, so it needs no Finder scripting and works the same on CI as on a Mac.
#
#   dmgbuild -s scripts/dmg.py -D app=target/release/Knoa.app Knoa Knoa.dmg

import os.path

app = defines["app"]  # noqa: F821
app_name = os.path.basename(app)

format = "UDZO"
files = [app]
symlinks = {"Applications": "/Applications"}
icon = os.path.join(app, "Contents", "Resources", "AppIcon.icns")

# A small window with only the app and Applications in it, side by side, so dragging one
# onto the other is the obvious thing to do. No background image: the window then follows
# light and dark mode, and the labels stay readable in both.
window_rect = ((200, 120), (540, 320))
default_view = "icon-view"
show_toolbar = False
show_sidebar = False
show_status_bar = False
show_pathbar = False
show_tab_view = False
icon_size = 128
text_size = 13
icon_locations = {app_name: (140, 140), "Applications": (400, 140)}
