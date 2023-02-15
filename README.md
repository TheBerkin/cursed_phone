# Cursed Phone

An audio-only game engine for the Raspberry Pi that emulates various styles of analog telephones. Not a VoIP service.

Phone numbers are assigned to scripts (known as "agents") rather than people-- simply call the number to call the script. Agents can also be scripted to call the phone back.

## Features

What you get out of the box:

* Emulation of rotary, touch-tone, and pay phones-- includes configurable GPIO inputs for switchhook, rotary dial, keypad, and coin triggers
* Lua scripting system
* GPIO pin access from Lua
* WAV/OGG multi-channel audio playback support
* Realistic call progress/DTMF tones
* Switchhook dialing (WIP)
* Comfort noise
* Intercept services
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


## Running

By default the Engine will use the configuration file `cursed_phone.conf` in the current working directory, but the file location can be overridden with the `CURSED_CONFIG_PATH` environment variable.