[中文](./README.md)

# YueXu Lunar Almanac

<!-- codex-github-rules:bilingual-summary -->
> **English summary**: A low-resource Windows desktop lunar calendar for the full year that updates daily.

---

A full-year lunar desktop calendar that updates every day. Updates and settings are handled by a native Rust program; there is no resident service, browser, WebView, or network connection.

![Dark theme preview](assets/preview-dark.png)

[View the light theme preview](assets/preview-light.png)

## Product Status

- Windows 10/11 x64
- Twelve months, lunar dates, leap months, and a highlighted current day
- Six built-in themes (dark, light, moonlit, pine ink, cinnabar, and mist blue), plus importable custom themes; dark is the default and the choice is remembered locally
- Updates at login and every day at 00:01; a missed midnight update is retried after the system resumes
- Wallpaper size follows the physical resolution of the current primary display to avoid unnecessary 4K rendering, scaling, and cropping
- Per-user installation with no administrator permission
- Rust + lunar_rust + resvg + tiny-skia for updates, without Chrome, Edge, or WebView
- Native Windows settings window with calendar preview, theme controls, live colors, margins, and import/export

Double-click YueXu Calendar on the desktop or Start menu to open settings. The process runs only when the user opens it; scheduled tasks and daily wallpaper updates do not create a window.

## Installation

Download YueXu-<version>-windows-x64.zip from Releases and run this in PowerShell:

~~~powershell
Set-ExecutionPolicy -Scope Process Bypass
.\Install-YueXu.ps1
~~~

The open-source package is not code-signed. Verify the SHA256 file attached to the Release, and follow your organization's policy if Windows SmartScreen warns on first launch.

After installation, the desktop and Start menu contain YueXu Calendar. The settings window previews the year, switches themes, edits 11 colors, changes margins, and imports or exports themes. Preview changes are immediate; only Apply to Desktop saves settings and updates the wallpaper. Upgrade installation keeps existing themes, while a first install starts with dark.

## Command Line

~~~powershell
# Update immediately with the remembered theme
Start-Process -FilePath .\LunarCalendar.exe -ArgumentList @('--update', '--quiet') -Wait

# Switch to and remember the light theme
Start-Process -FilePath .\LunarCalendar.exe -ArgumentList @('--update', '--theme', 'light') -Wait

# Switch to and remember the moonlit theme
Start-Process -FilePath .\LunarCalendar.exe -ArgumentList @('--update', '--theme', 'moonlit') -Wait

# Export a theme template, then import and apply it
Start-Process -FilePath .\LunarCalendar.exe -ArgumentList @('--export-theme', '.\my-theme.json') -Wait
Start-Process -FilePath .\LunarCalendar.exe -ArgumentList @('--theme-file', '.\my-theme.json') -Wait

# Generate an image without setting it as wallpaper
Start-Process -FilePath .\LunarCalendar.exe -ArgumentList @('--update', '--width', '1920', '--height', '1080', '--set-wallpaper=false') -Wait

# Open preview and settings
Start-Process -FilePath .\LunarCalendar.exe -ArgumentList @('--preview')
~~~

LunarCalendar.exe uses the GUI subsystem. Keep -Wait when scripts call update, import, or export so the next step starts after the operation completes.

Generated wallpapers and preferences are stored in %LOCALAPPDATA%\YueXu. The application does not read calendar accounts, upload data, or require an API key.

Every color in a custom theme must use #RRGGBB. Margins are stored as local layout preferences in %LOCALAPPDATA%\YueXu\settings.json and are not written into theme files. themes/moonlit-ink.json is an importable example; login and daily updates continue to use the imported theme.

## Architecture

| Layer | Implementation |
| --- | --- |
| Calendar and lunar dates | Rust with the offline lunar_rust calendar |
| Layout and PNG | Rust-generated SVG rasterized in process by resvg and tiny-skia |
| Windows integration | Native Win32 SystemParametersInfoW and Windows Task Scheduler |
| Preview and settings | On-demand native Win32 window using the same renderer as the wallpaper |

## Commercial Builds

The repository provides reproducible Windows release builds:

~~~powershell
.\scripts\Build-Release.ps1
~~~

The build produces the GUI executable, installer, uninstaller, ICO icon, SHA256 checksums, and versioned zip. See docs/商业化方案.md for release and pricing strategy and docs/发布清单.md for the pre-release checklist.

## Development

~~~powershell
cargo test --locked
cargo run -- --update --width 1920 --height 1080 --set-wallpaper=false
~~~

Rust runtime code is in native/. native/ui.rs implements the settings window, while native/calendar.rs drives both preview and wallpaper generation.

## Open Source

YueXu is released under the MIT License. The open-source edition may be used, modified, and redistributed. Commercial editions may provide signed installers, dedicated theme packs, batch deployment, and priority support. See CONTRIBUTING.md for contribution guidelines.
