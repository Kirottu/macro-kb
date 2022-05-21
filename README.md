# Macro-KB
This is a utility for creating a macro keyboard on linux. Any improvements/suggestions are welcome!

# Usage
This software has 2 main components, the root client and the userspace server. It operates with the root daemon started by, for example systemd in `systemd/system/macro-kb-root.service` and the userspace server started by some autostart method with your graphical session. There are 2 config files:
- /etc/macro-kb.conf:
    The config for the root client, specify the device you want to grab inputs of with the `DEVICE` keyword. If your keyboard has other input devices for stuff like action buttons, that you do not want to grab you can specify them in `DEVICES_NOLISTEN`.
- ~/.config/macro-kb.conf:
    The config for the userspace server, you can specify what the keys should do in this config. Commands that can be sent to the software itself are `EXIT` and `RELOAD`. `EXIT` quits the software and `RELOAD` reloads the user config.

# Config file format
The config file format is pretty simple, a definition of a device for example would be:
```
DEVICE = /dev/input/by-id/<your_device_name>
```

For stuff like the key configurations and the devices to grab but ignore, multi line definitions are also supported. They work like this:
```
DEVICES_NOLISTEN = <a device> \
<another device> \
<yet another device>
```
```
# Reload config
KEY_SYSRQ = RELOAD \
notify-send "Reloaded config" "Macro keyboard userspace server config reloaded"
```

And as you can see from the snippet above, comments are supported with the `#` symbol at the start of the line. If it is not at the first character of the line, it will not be detected.