# Changelog

> [中文](CHANGELOG.md)

## 0.4.4 - 2026-08-05

- Added four predefined themes: Moon Sea, Pine Smoke, Cinnabar, and Mist Blue.
  Theme switching, previewing, persistence, and daily updates use the same
  built-in palette.

## 0.4.3 - 2026-08-05

- Added top and bottom margin controls. All four layout sliders refresh the
  preview in real time and are used by daily updates after **Apply to Desktop**.

## 0.4.2 - 2026-08-05

- The color picker refreshes the settings-window preview while colors are
  adjusted. Cancelling restores the previous state; confirming still requires
  **Apply to Desktop** to save the settings and update the wallpaper.
- Added draggable left safe-zone and right-margin controls. The preview changes
  immediately with the parameters; margins persist and are used for daily
  updates after **Apply to Desktop**.

## 0.4.1 - 2026-08-05

- Fixed native-window message forwarding that lost `wParam`: close, minimize,
  maximize, and system-cursor behavior in the upper-right corner now work
  correctly.
- Confirming a custom color now saves the theme and updates the desktop
  immediately, without an additional **Apply to Desktop** action.

## 0.4.0 - 2026-08-05

- Rebuilt the settings experience as a native Win32 window; it no longer opens a
  browser or local web page.
- The native window uses the same Rust renderer as the wallpaper and provides a
  year preview, light and dark themes, full palette editing, import, export, and
  apply actions.
- Removed the legacy HTML, JavaScript, CSS, and `yuexu://` web protocol; upgrades
  automatically remove the protocol registration.
- Daily updates now generate wallpapers at the current primary display's physical
  resolution, reducing unnecessary rendering and avoiding system re-scaling or
  cropping.

## 0.3.0 - 2026-08-05

- Added export, import, and persistence for custom theme JSON; color input is
  restricted to safe `#RRGGBB` values.
- Added custom-theme color samples to the preview page. Imported themes are
  rendered into wallpapers immediately by the native application.
- Installation and development scripts preserve existing themes by default, so
  upgrades do not overwrite user palettes.

## 0.2.0 - 2026-08-05

- Rebuilt as a native Rust runtime, removing the Go and Chromium screenshot
  rendering paths.
- Uses `lunar_rust` to calculate the lunar calendar, leap months, and heavenly
  stems/earthly branches locally, without browser ICU data.
- Rust generates SVG and renders PNG in process through `resvg` / `tiny-skia`.
- Wallpaper updates no longer start Chrome, Edge, or WebView; only manual preview
  opens the local settings page on demand.
- Adjusted the year-grid layout to retain a safe zone for left-side desktop icons
  and a right margin.

## 0.1.0 - 2026-08-04

- Initial release: a 12-month lunar desktop calendar for the full year.
- Includes dark and light themes; dark is the default and the user's choice is
  remembered.
- Uses Chromium's built-in Chinese lunar-calendar data and supports leap months
  and the years 1900-2100.
- Provides a native Windows launcher that does not retain a browser process after
  generating the static wallpaper.
- Supports per-user installation, Desktop/Start Menu entry points, logon updates,
  and daily automatic updates.
- Provides versioned Windows x64 release packages, SHA256 checksums, and
  uninstall entries.
