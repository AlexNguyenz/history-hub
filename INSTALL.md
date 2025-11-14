# Installation Guide for macOS

## Step 1: Install the App

1. Open the DMG file
2. Drag "Claude Code History Hub" to your Applications folder

## Step 2: First Launch

Because this app is not signed with an Apple Developer certificate, macOS will block it on first launch.

### Method 1: Right-Click to Open (Recommended)

1. Go to Applications folder
2. Find "Claude Code History Hub"
3. **Right-click** (or Control+Click) on the app
4. Click **"Open"**
5. A dialog will appear, click **"Open"** again
6. The app will now run (and you won't need to do this again)

### Method 2: System Settings

If you see "App is damaged" error:

1. Open Terminal
2. Run this command:
   ```bash
   xattr -cr "/Applications/Claude Code History Hub.app"
   ```
3. Now you can open the app normally

### Method 3: Security & Privacy Settings

1. Try to open the app (it will be blocked)
2. Go to **System Settings** → **Privacy & Security**
3. Scroll down to find "Claude Code History Hub was blocked"
4. Click **"Open Anyway"**
5. Click **"Open"** in the confirmation dialog

## Troubleshooting

If you still have issues, please run this command in Terminal:

```bash
xattr -cr "/Applications/Claude Code History Hub.app"
```

Then try opening the app again.

---

**Note**: This security warning only appears because the app is not code-signed with an Apple Developer certificate. The app is safe to use.
