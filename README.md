# You're Muted BTW

This is a Windows-exclusive tool for alerting you when voice activity is detected while your Discord has your mic muted.

It creates a system tray icon.

When clicked, that system tray icon launches a Bevy application.

That Bevy application monitors Discord via Windows UI Automation APIs to detect the state of the mute button in the bottom left (default) and bottom center of the screen (fullscreen voice call).
