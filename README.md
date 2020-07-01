# Cursed Phone

This repository houses code for a hobby project of mine, the Cursed Phone.

![](https://i.imgur.com/HMyeW6v.jpg)

Unless you have an old phone on hand that's wired for use with a Raspberry Pi, you probably won't find this repo very useful. But if you do, boy do I have the program for you.

## What even is this?

It's a program that emulates various styles of telephones and allows you to write callable services. You can use it to implement anything from home automation to interactive art pieces. 

Here's a short list of feature highlights:

* Emulation of rotary, touch-tone, and pay phones via GPIO
* Lua-based scripting system
* Fully-configurable everything
* Realistic call progress/DTMF tones
* WAV/OGG audio playback support
* Switchhook dialing
* Vibration support
* Motion sensing support
* Intercept services
* Comfort noise
* Compatible with all Raspberry Pi models
* Mock GPIO interface for desktop testing via stdin

## Building

These instructions are written for Debian (Linux), but this should run in Windows too, as long as you don't have the `rpi` feature enabled.

### Prerequisites

You'll first need to install the ALSA library:

```sh
sudo apt install libasound2-dev
```

### Cargo

Then, with Cargo installed, run the following command:

```sh
# Build without GPIO support (use this if testing on Windows)
cargo build --release

# Build with GPIO support (use this for Raspberry Pi)
cargo build --release --features=rpi
```

### Cross-compilation

If you plan on cross-compiling, you'll need to install the appropriate build target to your Rust toolchain.

* For ARMv6 (Raspberry Pi 1 / Zero / Zero W), use `arm-unknown-linux-gnueabihf`.
* For ARMv7/8 (Raspberry Pi 2 / 3 / 4), use `armv7-unknown-linux-gnueabihf`.

Make sure you have an appropriate linker installed and that Cargo can find it.
You can do this by filling out and adding the following to your `.cargo/config` file:
```toml
[target.<your target triple here>]
linker = "<your linker path here>"
```

If compiling on the target system, this step is unnecessary. However, be warned that it will take *forever*.

### Move the built executable

If you'll be running this as a service, move the built executable (found somewhere in your `target` folder) to the project's root directory before use. 

## Known issues

Please note that the GPIO library used in this program is currently not fully compatible with the Raspberry Pi 4B.