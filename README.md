## rstatus - bar for tiling wms(i3, sway, etc)

[![Build Status](https://img.shields.io/github/actions/workflow/status/okeri/rstatus/ci.yml?branch=master)](https://github.com/okeri/rstatus/actions) [![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)


### Building
* cargo build --release

### Running 
* copy one of sample configs to actual config e.g.
**mkdir -p ~/.config/rstatus**
**cp samples/simple.yaml ~/.config/rstauts/config.yaml**
**./rstatus**
* if everything goes ok you could paste rstatus command to config  
of your tiling wm

### Dependencies
* wireless-tools(iwlib-dev on some distros)
* alsa(optional)
* pipewire(optional)
* pulseaudio(optional)


### Sample screenshots
![simple](samples/simple.png) 
![color_prefix](samples/color_prefix.png) 
![powerline](samples/powerline.png) 

### Default Block options
* interval - update interval in seconds
* signal - signal for update block
* name - block name
* separator_width -  width of separator after block
* custom_separator -  use custom symbol(s) for block separator
* color - foreground color ('#RRGGBB')
* bgcolor - background color ('#RRGGBB')
* prefix - prefix of value
* prefix_color - color of prefix if any
* suffix - suffix of value
* suffix_color - color of suffix
* invalid - string displayed if value is invalid. if invalid value is displayed, prefix and suffix are ignored.
* invalid_color - color of invalid value (red is default)
* threshold_fix - if set to true, suffix and prefix changing colors accorgind to thresholds values
* thresholds - change color of value depending on thresholds

### Extending rstatus via custom block
See one of samples for syntax.  
It asks from your binary/shell scripts for output. First line is for value, second is for color(optional)  
Please also note, custom block executes command in the main thread. That means you shoud not make network  
requests here. This could be implemented in async way, but it also means you have to detect network activity,  
failure handlers and so on. Instead please check systemd timers, you always could send unix signal(kill/pkill) to  
rstatus from process triggered by systemd.
