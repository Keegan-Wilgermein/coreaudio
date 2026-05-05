# coreaudio

A safe, idiomatic Rust wrapper around the macOS CoreAudio Hardware Abstraction Layer (HAL).

This crate provides typed access to audio devices, streams, and system-level audio objects, with compile-time guarantees around property access permissions and listener support.

## Features

- **Type-safe object model** — `AudioObject<System>`, `AudioObject<Device>`, and `AudioObject<Stream>` expose only the operations valid for each object type.
- **Compile-time property safety** — Properties carry phantom types encoding their value type, owning object, read/write access, and listenability. Attempting to write a read-only property or listen to a non-listenable one is a compile error.
- **Property listeners** — Subscribe to property changes with `add_listener`, then poll with `latest()`, collect with `all_since_last_check()`, or block with `block_until_change()`.
- **IO Procs** — Register audio render callbacks on devices with `add_io_proc` and control playback with `play()` / `pause()`.
- **Structured error handling** — All CoreAudio `OSStatus` codes are mapped to a typed `ErrorKind` enum with human-readable four-character-code formatting.
- **Format support** — Rich enums for audio format IDs (Linear PCM, AAC variants, ALAC, AC3, Opus, MP3, etc.), format flags, sample formats, and sample resampling utilities.

## Requirements

- macOS (the crate is gated with `#[cfg(target_os = "macos")]`)
- [`coreaudio-sys`](https://crates.io/crates/coreaudio-sys) for raw FFI bindings
- [`core-foundation`](https://crates.io/crates/core-foundation) for `CFString` handling

## Quick start

```rust
use coreaudio::{AudioObject, System, Scope, DEVICE_NAME};

fn main() -> Result<(), coreaudio::CoreAudioError> {
    let system = AudioObject::<System>::default();

    // List all output devices
    let devices = system.devices_with_scope(Scope::Output)?;

    for device in &devices {
        let name: String = device.get_property(DEVICE_NAME)?;
        println!("{}", name);
    }

    Ok(())
}
```

## Usage

### Querying device properties

```rust
use coreaudio::{
    AudioObject, System, Scope,
    DEVICE_NAME, DEVICE_UID, DEVICE_NOMINAL_SAMPLE_RATE,
    DEVICE_IS_ALIVE, DEVICE_BUFFER_FRAME_SIZE,
};

let system = AudioObject::<System>::default();
let device = system.current_device(Scope::Output)?;

let name: String = device.get_property(DEVICE_NAME)?;
let uid: String = device.get_property(DEVICE_UID)?;
let sample_rate: f64 = device.get_property(DEVICE_NOMINAL_SAMPLE_RATE)?;
let alive: bool = device.get_property(DEVICE_IS_ALIVE)?;

println!("{name} ({uid}) — {sample_rate} Hz, alive: {alive}");
```

### Setting writable properties

```rust
use coreaudio::{AudioObject, System, Scope, DEVICE_NOMINAL_SAMPLE_RATE, DEVICE_BUFFER_FRAME_SIZE};

let system = AudioObject::<System>::default();
let device = system.current_device(Scope::Output)?;

device.set_property(DEVICE_NOMINAL_SAMPLE_RATE, 48000.0)?;
device.set_property(DEVICE_BUFFER_FRAME_SIZE, 512)?;
```

### Listening for property changes

```rust
use coreaudio::{AudioObject, System, Scope, DEVICE_NOMINAL_SAMPLE_RATE};
use std::time::Duration;

let system = AudioObject::<System>::default();
let device = system.current_device(Scope::Output)?;

let listener = device.add_listener(DEVICE_NOMINAL_SAMPLE_RATE)?;

// Non-blocking check
if let Some(new_rate) = listener.latest() {
    println!("Sample rate changed to {new_rate}");
}

// Blocking with timeout
match listener.block_for_duration(Duration::from_secs(5)) {
    Ok(rate) => println!("Changed to {rate}"),
    Err(e) => println!("Timed out or error: {e}"),
}
```

### Working with streams

```rust
use coreaudio::{AudioObject, System, Scope, STREAM_NAME, STREAM_IS_ACTIVE, STREAM_DIRECTION};

let system = AudioObject::<System>::default();
let device = system.current_device(Scope::Output)?;
let streams = device.streams_with_scope(Scope::Output)?;

for stream in &streams {
    let format = stream.stream_virtual_format()?;
    println!(
        "Format: {:?}, Sample rate: {}, Channels: {}",
        format.format_id(),
        format.sample_rate(),
        format.channels_per_frame(),
    );
}
```

### Registering an IO proc (audio callback)

```rust
use coreaudio::{AudioObject, System, Scope};

let system = AudioObject::<System>::default();
let device = system.current_device(Scope::Output)?;

let io_proc = device.add_io_proc(|buffers| {
    for buffer in buffers {
        // Fill with silence
        buffer.data.fill(0.0);
    }
})?;

io_proc.play()?;
// ...
io_proc.pause()?;
io_proc.remove();
```

### Available buffer sizes and sample rates

```rust
let buffer_range = device.avaliable_buffer_sizes()?;
println!("Valid buffer sizes: {:?}", buffer_range.valid_sizes());

let sample_rates = device.avaliable_sample_rates()?;
for range in &sample_rates {
    println!("{}–{} Hz", range.min(), range.max());
}
```

## Property reference

### Device properties

| Constant | Type | Access | Listenable |
|---|---|---|---|
| `DEVICE_NAME` | `String` | Read | No |
| `DEVICE_UID` | `String` | Read | No |
| `DEVICE_IS_ALIVE` | `bool` | Read | Yes |
| `DEVICE_IS_RUNNING` | `bool` | Read | Yes |
| `DEVICE_NOMINAL_SAMPLE_RATE` | `f64` | Read/Write | Yes |
| `DEVICE_BUFFER_FRAME_SIZE` | `u32` | Read/Write | Yes |
| `DEVICE_INPUT_LATENCY` | `u32` | Read | No |
| `DEVICE_OUTPUT_LATENCY` | `u32` | Read | No |
| `DEVICE_HOG_MODE` | `i32` | Read/Write | Yes |

### Stream properties

| Constant | Type | Access | Listenable |
|---|---|---|---|
| `STREAM_NAME` | `String` | Read | No |
| `STREAM_IS_ACTIVE` | `bool` | Read | Yes |
| `STREAM_DIRECTION` | `u32` | Read | No |
| `STREAM_LATENCY` | `u32` | Read | No |

### System properties

| Constant | Type | Access | Listenable |
|---|---|---|---|
| `SYSTEM_NAME` | `String` | Read | No |
| `SYSTEM_IS_INITING_OR_EXITING` | `bool` | Read | No |
| `SYSTEM_SLEEPING_IS_ALLOWED` | `bool` | Read/Write | Yes |
| `SYSTEM_POWER_HINT` | `u32` | Read/Write | No |

## Error handling

All fallible operations return `Result<T, CoreAudioError>`. The error type wraps a typed `ErrorKind` enum and the raw `OSStatus` code. You can match on the kind or inspect the four-character code string:

```rust
use coreaudio::ErrorKind;

match device.get_property(DEVICE_NAME) {
    Ok(name) => println!("{name}"),
    Err(e) => match e.kind() {
        ErrorKind::BadDevice => println!("Invalid device"),
        ErrorKind::Permissions => println!("Device is hogged by another process"),
        _ => println!("Error: {e}"),
    }
}
```

## Supported audio formats

The `FormatId` enum covers Linear PCM, AAC (Standard, HE, HEv2, LD, ELD, ELDv2, ELD+SBR, Spatial), Apple Lossless, AC-3, Enhanced AC-3, APAC, AES3, A-Law, AMR, AMR-WB, Opus, and MP3. Unrecognised format IDs are preserved as `FormatId::Unknown(u32)`.

## Roadmap

- **Sample rate validation** — `SampleRateRange` currently exposes raw min/max values. A future release will add a validation method that checks whether a given sample rate falls within a device's supported ranges and snaps to the nearest valid rate.
- Dedicated `AudioObject<Clock>`,  `AudioObject<Box>`, and `AudioObject<Tap>` with unique methods

## Breaking changes - v0.2.0
- A lot of property rules have been updated to be more accurate
- Function `.add_io_proc()` returns `&[AudioBuffer]` instead of `&mut [AudioBuffer]`<br>The data stored inside is still mutable
- Dedicated methods for
    - `.avaliable_sample_rates()`
    - `.avaliable_buffer_sizes()`
    - `.stream_virtual_format()`
    - `stream_physical_format()`

    have been removed in favour of them being accessed via `.get_property()`
- Some properties now require calls to `.with_qualifier()` and / or `.for_element()`
- Some properties now return wrapper types instead of the raw values but they all implement `.into()` or have dedicated reverse functions
- Properties have a 5th type parameter instead of the previous 4

## Disclaimer
Apple's documentation on what properties can be listened to is pretty much non existant.
Because of this, pretty much all writable properties have been made listenable and it will return an error if it turns out not to be.

If you know of any documentation or if a specific property is incorrectly set, please make an issue in the repository and I will fix it at my earliest convenience.

## License

See LICENSE file for details.