# Cursed Phone

This repository houses code for a hobby project of mine, the Cursed Phone.

The Cursed Phone is a repurposed rotary phone with a Raspberry Pi as its brain. It's not a "real" telephone; it instead does whatever I want through the convenient and universally understood interface of a rotary dial.

## What is this?

It's a sort of audio-only game engine for the Raspberry Pi that emulates various styles of telephones. Phone numbers are assigned to scripts (known as "agents") rather than people-- call the number to call the script. You can even make your scripts call the phone. Imagine the possibilities!

What you get out of the box:

* Emulation of rotary, touch-tone, and pay phones via GPIO
* Lua scripting system
* Fully-configurable everything
* Realistic call progress/DTMF tones
* WAV/OGG audio playback support
* GPIO access in scripts
* Switchhook dialing (WIP)
* Intercept services
* Comfort noise
* Mock GPIO interface for desktop testing via stdin
* Compatibility with all Raspberry Pi models

## Building

These instructions are written for Debian (Linux), but this should run in Windows too, as long as you don't have the `rpi` feature enabled.

### Prerequisites

Before building for Linux, you'll first need to install the ALSA library:

```sh
sudo apt install libasound2-dev
```

### Cargo

Then, with Cargo installed, run the appropriate build command for your platform:

```sh
# Build with GPIO support (for Raspberry Pi)
cargo build --release --features=rpi

# Build without GPIO support (for non-RPi platforms)
cargo build --release
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

If you'll be running this as a service, move the built executable (found in `/target/release`) to the project's root directory before use. 