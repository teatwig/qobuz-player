# qobuz-player

### High resolution audio player backed by Qobuz

Powered by [Qobuz](https://www.qobuz.com). Requires a paid subscription. This does not allow you to listen for free.

The web interface is ideal for a setup with a single board conputer, raspberry pi, connected to the speaker system and controlled with a smartphone or tablet.

<img src="/qobuz-player-webui.png?raw=true" width="240">

## Player Features

- [GStreamer](https://gstreamer.freedesktop.org/)-backed player
- High resolution audio: Supports up to 24bit/192Khz (max quality Qobuz offers)
- MPRIS support (control via [playerctl](https://github.com/altdesktop/playerctl) or other D-Bus client)
- Gapless playback
- Web UI 

## Requirements

- [GStreamer v1.18+](https://gstreamer.freedesktop.org/documentation/installing/index.html) (comes with most/all current Linux)

## Installation

### Download Release

Download the tar.gz file for your supported OS from the releases page, extract the file and execute `qobuz-player` or copy it to the your `$PATH`.

### Build from source

On Debian, Arch and Fedora, `just build-player` should make a reasonable effort to install the necessary dependencies needed to build the app and then build it.

## Get started

Run `qobuz-player --help` or `qobuz-player <subcommand> --help` to see all available options.

To get started:

```shell
qobuz-player config username # enter username at prompt
qobuz-player config password # enter password at prompt

# open player
qobuz-player open

# open player with web ui
qobuz-player --web open
```

## Web UI

The player can start an embedded web interface. This is disabled by default and must be started with the `--web` argument. It also listens on `0.0.0.0:9888` by default,
but an inteface can be specified with the `--interface` argument.

Go to `http://<ip>:9888` to view the UI.

## Credits
Qobuz-player started as a fork of [hifi.rs](https://github.com/iamdb/hifi.rs) but has since diverged. 
Qobuz-player is mainly focused on the setup where a single boatd computer is connected to a speaker system.
