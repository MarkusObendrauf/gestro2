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
        ╲    |    ╱
    Up-Left  |  Up-Right
          ╲  |  ╱
  Left ────  ●  ──── Right
          ╱  |  ╲
  Down-Left  |  Down-Right
        ╱    |    ╲
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

### macOS: Setup

**Opening the app for the first time:** Gestro is not notarized by Apple (it's open-source and free). macOS may refuse to open it. To get around this:
1. Open **Terminal** and run: `xattr -cr /Applications/Gestro.app`
2. Then launch Gestro normally from your Applications folder

**Granting Accessibility permission:** Gestro needs Accessibility access to intercept mouse events. On first launch, macOS should prompt you. If it doesn't, or if gestures aren't working:
1. Open **System Settings > Privacy & Security > Accessibility**
2. Click the **+** button and add **Gestro.app** from your Applications folder
3. Make sure the toggle next to Gestro is **on**
4. Quit and relaunch Gestro

If you still have issues after granting permission, try resetting it:
```
tccutil reset Accessibility com.gestro.gestro
```
Then relaunch Gestro and re-add it in Accessibility settings.

**Note:** Always launch Gestro from the Applications folder (or Spotlight), not by running the binary directly — macOS ties Accessibility permissions to the `.app` bundle path.

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
| Left | Previous tab | `Ctrl+[` |
| Right | Next tab | `Ctrl+]` |
| Up | New tab | `Ctrl+n` |
| Down | Close tab | `Ctrl+w` |
| Up-Left | Forward | `BrowserForward` |
| Up-Right | Back | `BrowserBack` |
| Down-Right | Refresh | `F5` |
| Down-Left | Re-open closed tab | `Ctrl+Shift+t` |

These are just suggestions — you can bind any keyboard shortcut to any direction.

## Troubleshooting

**Gestro doesn't respond to gestures**
- On Linux, make sure you're in the `input` group (see [Permission Setup](#linux-permission-setup))
- On macOS, make sure Accessibility permission is granted (see [macOS: Setup](#macos-setup))
- Check the tray icon — if there's an error, the settings window will show details

**macOS: "Gestro.app is damaged and can't be opened"**
- This is Gatekeeper blocking an unsigned app. Run `xattr -cr /Applications/Gestro.app` in Terminal, then try again.

**macOS: Gestures still don't work after granting Accessibility**
- Quit Gestro, run `tccutil reset Accessibility com.gestro.gestro` in Terminal, then relaunch and re-add it in System Settings > Privacy & Security > Accessibility.
- Make sure you're launching from `/Applications`, not running the binary directly.

**Tray icon doesn't appear**
- Some desktop environments need a tray/appindicator extension. On GNOME, install the [AppIndicator extension](https://extensions.gnome.org/extension/615/appindicator-support/).

## License

[MIT](LICENSE) — Markus Obendrauf
