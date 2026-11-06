# Verifying

`make check` passing is not done. Done is having seen the change work in the app.

## Run and check

1. `make dev` in the background. It runs Knoa Dev, the debug build as its own app, and
   logs at debug level.
2. Drive Knoa Dev (`dev.knoa.Knoa-Dev`) with the computer-use tools, never an installed
   Knoa: open the screen you changed and do what a user would do.
3. Check the happy path and the one failure most likely to happen (no network, empty
   library, bad input).
4. Read the log for warnings and errors your change caused.
5. Fix, rerun, repeat until it works. Never open a PR on a change you haven't seen work.

If the change has no visible effect, show it some other way: logs, the database, a
command's output. Say in the PR what you checked.

## Screenshots

Needed for any UI change. With the app running, capture only the Knoa Dev window:

```bash
.agents/skills/knoa-standards/scripts/screenshot.sh shot.png
```

Never capture the whole screen or a screen region: other windows may show private
content. Look at the image before using it.

- Show the state the change is about, with realistic data.
- Before and after, if the change alters an existing screen.
- Light and dark mode if the change touches colors.
