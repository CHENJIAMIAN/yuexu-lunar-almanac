# Contributing Guide

> [中文](CONTRIBUTING.md)

Issues, design suggestions, and Pull Requests are welcome.

## Local verification

```powershell
cargo test --locked
cargo run -- --update --width 1920 --height 1080 --set-wallpaper=false
```

For visual changes, inspect both dark and light themes and output at 1920×1080
and 3840×2160. For lunar-calendar logic, cover Chinese New Year, leap months,
and dates crossing Gregorian calendar years. Wallpaper rendering must remain
browser-independent.

## Commit conventions

- Use Chinese for commit messages and user-visible copy.
- Do not introduce external network requests, telemetry, or advertising SDKs.
- Do not commit build artifacts in `output/`, `dist/`, or `release/`.
