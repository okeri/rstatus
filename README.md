## rstatus - bar for tiling wms(i3, sway, etc)

[![Build Status](https://img.shields.io/github/actions/workflow/status/okeri/rstatus/ci.yml?branch=master)](https://github.com/okeri/rstatus/actions) [![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

rstatus feeds the i3bar protocol on stdout, so it works as `status_command` for i3, sway
and anything else speaking that protocol.

### Building
* `cargo build --release` - builds with the default **pipewire** backend

Sound backends are selected by cargo features. Exactly one is not required - every
enabled backend is compiled in, and the first one that connects at runtime is used
(pipewire, then pulse, then alsa).

* `cargo build --release --features alsa` - pipewire + alsa
* `cargo build --release --no-default-features --features pulse` - pulseaudio only
* `cargo build --release --no-default-features` - no sound support, the volume block
always renders its `invalid` value

### Dependencies
* pipewire (default)
* alsa (optional)
* pulseaudio (optional)

### Running
* copy one of the sample configs to the actual config, e.g.
**mkdir -p ~/.config/rstatus**
**cp samples/simple.yaml ~/.config/rstatus/config.yaml**
**./rstatus**
* if everything goes ok you could paste rstatus command to config
of your tiling wm

The config path is fixed: **$HOME/.config/rstatus/config.yaml**. There are no command
line options.

### Configuration
The config is a YAML sequence of blocks. Each entry is tagged with the block type and
carries its options; blocks are rendered left to right in the order they are listed:

```yaml
  - !temperature
      sensor: 'x86_pkg*'
      suffix: ' °C'
      interval: 3

  - !time
      format: '%H:%M'
      interval: 1
```

Available block types: **!battery**, **!cpuload**, **!custom**, **!filesystem**,
**!memory**, **!network**, **!temperature**, **!time**, **!volume**.

Unknown options are silently ignored, so a typo in an option name costs you the option
without any warning.

### Updating blocks
* **interval** - update period in seconds. `0` (the default) means the block is never
updated by the timer, only by a signal.
* **signal** - offset from `SIGRTMIN` (34). `0` (the default) disables signal updates.
With `signal: 3` the block is refreshed by `pkill -RTMIN+3 rstatus`.

At least one block must have a non-zero `interval`, otherwise rstatus prints nothing at
all and exits immediately. The timer resolution is the greatest common divisor of all
non-zero intervals.

### Common block options
Every block accepts these:

* **interval** - update interval in seconds (see above)
* **signal** - signal for updating the block (see above)
* **separator_width** - width in pixels of the separator drawn after the block
* **custom_separator** - custom symbol(s) drawn *before* the block instead of the
regular separator, used for powerline style bars. It is only rendered when **bgcolor**
is also set: the symbol is painted in this block's `bgcolor` on top of the previous
block's background. Setting it also suppresses `separator_width` for this block.
* **color** - foreground color of the value, '#RRGGBB' or 'RRGGBB' (default '#FFFFFF')
* **bgcolor** - background color of the whole block (default: none)
* **prefix** - text placed before the value
* **prefix_color** - color of the prefix (defaults to the current value color)
* **suffix** - text placed after the value
* **suffix_color** - color of the suffix (defaults to the current value color)
* **invalid** - string displayed when the value is invalid (default 'invalid'). While it
is displayed, prefix and suffix are not rendered.
* **invalid_color** - color of the invalid string (default '#FF0000')
* **threshold_fix** - if true, prefix and suffix follow the threshold color instead of
`prefix_color`/`suffix_color`, but only while that threshold color actually differs from
`color`
* **thresholds** - map of `lower bound: color`. The color of the highest bound that is
less than or equal to the value wins; below the lowest bound `color` is used. Only
numeric values have thresholds - blocks producing text ignore them.

Note that the block name reported in the i3bar protocol is the block type
(`temperature`, `volume`, ...) and cannot be configured.

Some blocks inject their own prefix/suffix (battery statuses, network, volume jack
icons). Those are rendered *between* your `prefix`/`suffix` and the value, so both are
visible at once.

### Blocks

#### !battery
* **sensor** (required) - power supply directory, e.g. '/sys/class/power_supply/BAT0'.
`status` and `capacity` are read from it.
* **statuses** - per-state decoration, each with its own `prefix` and `suffix`:
  * **online** - charging
  * **offline** - discharging
  * **full** - any other state reported by the kernel
* **warning_level** - capacity percentage below which `warning_action` fires (default 0,
i.e. never)
* **warning_action** - shell command executed while discharging and below
`warning_level`. It runs on every update, synchronously, so keep it short.

The value is the battery capacity in percent; a missing or unreadable sensor renders
`invalid`.

#### !cpuload
No options besides the common ones. The value is the busy CPU percentage since the
previous update, so the very first update always renders `invalid`.

#### !filesystem
* **path** (required) - any path on the filesystem you want to measure, e.g. '/home'

The value is used space in percent, rounded up.

#### !memory
No options besides the common ones. The value is used memory in percent, computed as
`100 - MemAvailable / MemTotal` from /proc/meminfo.

#### !network
Reports on the interface holding the default route with the lowest metric.

* **wifi** - prefix used when that interface is wireless (default 'wifi'). The value is
then the signal strength in percent, with '%' appended automatically.
* **ethernet** - text displayed as the value when the interface is not wireless
(default 'eth')

Without a default route the block renders `invalid`.

#### !temperature
* **sensor** (required) - sensor name or name mask, e.g. 'x86_pkg_temp' or 'x86_pkg*'.
Masks support `*` (any sequence of characters) and `?` (exactly one character); a mask
without wildcards is an exact name match.

Names are matched against **/sys/class/thermal/\*/type**, **/sys/class/hwmon/\*/name**
and the hwmon **temp\*_label** files, so chips exposing no thermal zone (coretemp,
k10temp, nvme, amdgpu) are covered too. When a mask matches several sensors, the highest
temperature among them is displayed - 'coretemp\*' therefore shows the hottest core.
The value is in degrees Celsius. Sysfs paths are not accepted, use names instead.

#### !time
* **format** - chrono/strftime format string (default '%d.%m.%Y %H:%M')

#### !volume
The backend is chosen at runtime: pipewire, then pulseaudio, then alsa, limited to the
features the binary was built with. The block refreshes itself on backend events, so it
does not need an `interval`.

* **mixer** - alsa simple mixer element name (default 'PCM'). If a 'Master' element
exists, muting Master renders `invalid` and the displayed level is taken from **mixer**.
Pipewire and pulseaudio always report the default sink volume and ignore this option.
* **card** - alsa card name (default 'default'). Ignored by pipewire and pulseaudio.
* **jack_icons** - list of two strings, `[plugged, unplugged]`, used as an icon in front
of the value. Lists shorter than two entries are ignored.
* **jack_only** - list of sink names that are always treated as "jack plugged", useful
for outputs with no jack detection. The sink name is `node.nick` on pipewire, the
default sink name on pulseaudio, and the card name on alsa.
* **alsa_jack_switch_outputs** - alsa only. On plug mute 'Speaker' and unmute
'Headphone', on unplug do the opposite (default false).
* **alsa_jack_mute_on_unplug** - alsa only. Mute 'Master' when the jack is unplugged
(default false).
* **alsa_jack_unmute_on_plug** - alsa only. Unmute 'Master' when the jack is plugged
(default false).

A muted output renders `invalid`, which is how the samples display a "muted" indicator.

#### !custom
* **command** (required) - shell command executed via `sh -c`

The first line of stdout is the value: it becomes a number if it parses as one
(thresholds then apply), otherwise it is used as text. The optional second line sets the
value color ('#RRGGBB' or 'RRGGBB'); once set it replaces `color` for good. Empty output
renders `invalid`.

### Extending rstatus via custom block
See one of samples for syntax.
It asks from your binary/shell scripts for output. First line is for value, second is for color(optional)
Please also note, custom block executes command in the main thread. That means you shoud not make network
requests here. This could be implemented in async way, but it also means you have to detect network activity,
failure handlers and so on. Instead please check systemd timers, you always could send unix signal(kill/pkill) to
rstatus from process triggered by systemd.

### Sample screenshots
![simple](samples/simple.png)
![color_prefix](samples/color_prefix.png)
![powerline](samples/powerline.png)
