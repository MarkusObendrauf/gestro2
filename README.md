# Gestro

**Launch anything with a flick of your mouse.**

Gestro is a lightweight, system-wide mouse gesture tool. Hold down your right mouse button, drag in a direction, and release to instantly trigger a keyboard shortcut — like switching desktops, opening apps, controlling media, or anything else you can bind to a key combo.

It lives quietly in your system tray and works across your entire desktop — no matter which app is in focus.

## How It Works

1. **Hold** the right mouse button anywhere on screen
2. **Drag** in one of 8 directions (up, down, left, right, or diagonals)
3. **Release** to fire the shortcut you've assigned to that direction

If you right-click without dragging (or drag less than the threshold distance), Gestro passes through a normal right-click — so your regular right-click menu still works.

```
            Up
        ╲   |   ╱
    Up-Left  |  Up-Right
          ╲  |  ╱
  Left ────  ●  ──── Right
          ╱  |  ╲
  Down-Left  |  Down-Right
        ╱   |   ╲
           Down
```

## Supported Platforms

| Platform | Status |
|----------|--------|
| Linux (X11 & Wayland) | Supported |
| macOS | Supported |
| Windows | Supported |

## Installation

Download the latest release for your platform from the [Releases](https://github.com/mobendrauf/gestro2/releases) page:

| Platform | Download |
|----------|----------|
| Linux (Debian/Ubuntu) | `.deb` |
| Linux (Fedora/openSUSE) | `.rpm` |
| Linux (universal) | `.AppImage` |
| macOS | `.dmg` |
| Windows | `.msi` or `.exe` |

### Linux: Permission Setup

On Linux, Gestro needs access to your input devices. Your user account must be in the `input` group:

```
sudo usermod -aG input $USER
```

Then **log out and back in** for the change to take effect.

If Gestro can't access input devices, it will show an error in the settings window.

## Usage

After installing, Gestro starts in the system tray (the small icon area near your clock).

### Opening Settings

Click the tray icon and select **Settings** to open the configuration window.

### Assigning Gestures

In the settings window, you'll see a wheel with 8 directions. Click any direction to assign a keyboard shortcut to it. Press the key combination you want (e.g., `Ctrl+Alt+Right` for "next desktop") and confirm.

### Adjusting Sensitivity

Use the **threshold slider** in settings to control how far you need to drag before a gesture registers. A lower value makes gestures more sensitive; a higher value means you need to drag further.

### Start at Login

Enable **Launch at login** in settings so Gestro runs automatically when you start your computer.

### Quitting

Right-click the tray icon and select **Quit**.

## Example Gestures

Here are some ideas for what to bind:

| Direction | Action | Typical Shortcut |
|-----------|--------|-----------------|
| Left | Previous desktop | `Ctrl+Alt+Left` |
| Right | Next desktop | `Ctrl+Alt+Right` |
| Up | Mission Control / Overview | `Super` |
| Down | Show desktop | `Super+D` |
| Up-Right | New browser tab | `Ctrl+T` |
| Down-Right | Close tab | `Ctrl+W` |
| Up-Left | Undo | `Ctrl+Z` |
| Down-Left | Volume mute | `AudioMute` |

These are just suggestions — you can bind any keyboard shortcut to any direction.

## Troubleshooting

**Gestro doesn't respond to gestures**
- On Linux, make sure you're in the `input` group (see [Permission Setup](#linux-permission-setup))
- Check the tray icon — if there's an error, the settings window will show details

**Right-click menu stopped working**
- This shouldn't happen. If you right-click without dragging (or drag only a tiny amount), Gestro passes through a normal right-click. Try increasing the threshold in settings if you're accidentally triggering gestures.

**Tray icon doesn't appear**
- Some desktop environments need a tray/appindicator extension. On GNOME, install the [AppIndicator extension](https://extensions.gnome.org/extension/615/appindicator-support/).

## License

[MIT](LICENSE) — Markus Obendrauf
