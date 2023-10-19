# sundial

[`org.freedesktop.timedate1`](https://www.freedesktop.org/software/systemd/man/org.freedesktop.timedate1.html) D-Bus daemon and client for non-systemd systems.
`org.freedesktop.timedate1` is the D-Bus interface used by systemd's `timedated` and `timedatectl`.
Many desktop environments also use this D-Bus interface as the backend for their time and date settings UIs:

| Desktop Environment | Uses `timedate1` D-Bus Interface? | `sundial` Tested? | Notes |
|:--------------------|:---------------------------------:|:-----------------:|:------|
| Budgie       | ✅ | ❔ | |
| Cinnamon     | ✅ | ❔ | |
| Cutefish     | ✅ | ❔ | |
| Deepin       | ✅ | ❔ | has an alternative interface that does nothing (set at build-time) |
| GNOME 3      | ✅ | ❔ | |
| KDE Plasma 5 | 🟨 | ❔ | has "legacy" methods to fall back to if the D-Bus interface is unavailable |
| LXDE         | ❌ | -  | |
| LXQt         | ✅ | ❔ | |
| MATE         | ✅ | ❔ | |
| Pantheon     | ✅ | ❔ | |
| Xfce         | ❌ | -  | |

> ***Key:** ✅: Required/Works, 🟨: Optional, ❌: Not used/Does not work, ❔: Untested*

## Installation

with [rinstall](https://github.com/danyspin97/rinstall):
```
# rinstall install --system
# rinstall install --system --yes
```

## Usage

`sundiald` will be launched automatically when requested over D-Bus.
`sundialctl` is a small utility to interface with `sundiald` from the command-line.
See `sundialctl --help` for more information.

## Copyright

Copyright (c) 2023, classabbyamp  
Released under the BSD-2-Clause licence
