# coreaudio

A safe, idiomatic Rust wrapper around the macOS CoreAudio Hardware Abstraction Layer (HAL).

This crate provides typed access to audio devices, streams, and system-level audio objects, with compile-time guarantees around property access permissions and listener support.

## Features

- **Type-safe object model** — `AudioObject<System>`, `AudioObject<Device>`, and `AudioObject<Stream>` expose only the operations valid for each object type.
- **Compile-time property safety** — Properties carry phantom types encoding their value type, owning object, read/write access, and listenability. Attempting to write a read-only property or listen to a non-listenable one is a compile error.
- **Property builder methods** — Properties that require an element (channel) or qualifier data expose `.for_element(n)` and `.with_qualifier(value)` builder methods. Forgetting to call them is a compile error.
- **Property listeners** — Subscribe to property changes with `add_listener`, then poll with `latest()`, drain with `all_since_last_check()`, or block with `block_until_change()` / `block_for_duration()`.
- **IO Procs** — Register audio render callbacks on devices with `add_io_proc` and control playback with `play()` / `pause()`.
- **Structured error handling** — All CoreAudio `OSStatus` codes are mapped to a typed `ErrorKind` enum with human-readable four-character-code formatting.
- **Format support** — Rich enums for audio format IDs (Linear PCM, AAC variants, ALAC, AC3, Opus, MP3, etc.), format flags, sample formats, transport types, terminal types, and sample resampling utilities.

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
    DEVICE_NAME, DEVICE_UID, DEVICE_NOMINAL_SAMPLE_RATE, DEVICE_IS_ALIVE,
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
device.set_property(DEVICE_BUFFER_FRAME_SIZE, 512u32)?;
```

### Listening for property changes

```rust
use coreaudio::{AudioObject, System, Scope, DEVICE_NOMINAL_SAMPLE_RATE};
use std::time::Duration;

let system = AudioObject::<System>::default();
let device = system.current_device(Scope::Output)?;

let listener = device.add_listener(DEVICE_NOMINAL_SAMPLE_RATE)?;

// Non-blocking — returns the most recent change, or None
if let Some(new_rate) = listener.latest() {
    println!("Sample rate changed to {new_rate}");
}

// Blocking with timeout
match listener.block_for_duration(Duration::from_secs(5)) {
    Ok(rate) => println!("Changed to {rate}"),
    Err(e) => println!("Timed out or error: {e}"),
}
```

### Properties that require an element or qualifier

Some properties target a specific channel (element) or need a qualifier value before they can be used. Call `.for_element()`, `.with_qualifier()`, or both — in either order — to complete the property. Forgetting is a compile error.

```rust
use coreaudio::{
    AudioObject, System, Scope,
    DEVICE_VOLUME_SCALAR, DEVICE_MUTE,
    DEVICE_DATA_SOURCE, DEVICE_DATA_SOURCE_NAME,
    MissingElement, MissingQualifier,
};

let system = AudioObject::<System>::default();
let device = system.current_device(Scope::Output)?;

// Element-only: read the volume of channel 1
let volume: f32 = device.get_property(DEVICE_VOLUME_SCALAR.for_element(1))?;

// Element-only: mute channel 1
device.set_property(DEVICE_MUTE.for_element(1), true)?;

// Element-only: read the active data source on channel 1
let source_id: u32 = device.get_property(DEVICE_DATA_SOURCE.for_element(1))?;

// Element + qualifier: look up the name of that source
let source_name: String = device.get_property(
    DEVICE_DATA_SOURCE_NAME
        .for_element(1)
        .with_qualifier(source_id)
)?;
println!("Active source: {source_name}");
```

### Working with streams

```rust
use coreaudio::{AudioObject, System, Scope, STREAM_VIRTUAL_FORMAT, STREAM_NAME};

let system = AudioObject::<System>::default();
let device = system.current_device(Scope::Output)?;
let streams = device.streams_with_scope(Scope::Output)?;

for stream in &streams {
    let name: String = stream.get_property(STREAM_NAME)?;
    let format = stream.get_property(STREAM_VIRTUAL_FORMAT)?;
    println!(
        "{name}: {:?}, {} Hz, {} ch",
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

let mut io_proc = device.add_io_proc(|buffers| {
    for buffer in buffers {
        buffer.data.fill(0.0); // silence
    }
})?;

io_proc.play()?;
// ... render audio ...
io_proc.pause()?;
io_proc.remove();
```

### Available buffer sizes and sample rates

```rust
use coreaudio::{DEVICE_BUFFER_FRAME_SIZE_RANGE, DEVICE_AVAILABLE_SAMPLE_RATES};

let buffer_range = device.get_property(DEVICE_BUFFER_FRAME_SIZE_RANGE)?;
println!("Valid buffer sizes: {:?}", buffer_range.valid_sizes());

let sample_rates = device.get_property(DEVICE_AVAILABLE_SAMPLE_RATES)?;
for range in &sample_rates {
    println!("{}–{} Hz", range.min(), range.max());
}
```

## Property reference

Properties marked **element** require `.for_element(channel)` before use.  
Properties marked **qualifier** require `.with_qualifier(value)` before use.  
Properties marked **both** require both calls (in either order).

### Object properties (all object types)

| Constant | Type | Access | Listenable | Extra |
|---|---|---|---|---|
| `OBJECT_BASE_CLASS` | `u32` | Read | No | — |
| `OBJECT_CLASS` | `u32` | Read | No | — |
| `OBJECT_OWNER` | `u32` | Read | No | — |
| `OBJECT_MODEL_NAME` | `String` | Read | No | — |
| `OBJECT_MANUFACTURER` | `String` | Read | No | — |
| `OBJECT_CREATOR` | `String` | Read | No | — |
| `OBJECT_ELEMENT_NAME` | `String` | Read | No | element |
| `OBJECT_ELEMENT_CATEGORY_NAME` | `String` | Read | No | element |
| `OBJECT_ELEMENT_NUMBER_NAME` | `String` | Read | No | element |
| `OBJECT_OWNED_OBJECTS` | `Vec<u32>` | Read | No | qualifier: `Vec<u32>` |

### Device properties


| Constant | Type | Access | Listenable | Extra |
|---|---|---|---|---|
| `DEVICE_NAME` | `String` | Read | Yes | — |
| `DEVICE_UID` | `String` | Read | No | — |
| `DEVICE_MODEL_UID` | `String` | Read | No | — |
| `DEVICE_CONFIGURATION_APPLICATION` | `String` | Read | No | — |
| `DEVICE_TRANSPORT_TYPE` | `TransportType` | Read | No | — |
| `DEVICE_IS_ALIVE` | `bool` | Read | Yes | — |
| `DEVICE_IS_RUNNING` | `bool` | Read | Yes | — |
| `DEVICE_IS_HIDDEN` | `bool` | Read | No | — |
| `DEVICE_CAN_BE_DEFAULT` | `bool` | Read | No | — |
| `DEVICE_CAN_BE_DEFAULT_SYSTEM` | `bool` | Read | No | — |
| `DEVICE_NOMINAL_SAMPLE_RATE` | `f64` | Read/Write | Yes | — |
| `DEVICE_AVAILABLE_SAMPLE_RATES` | `Vec<SampleRateRange>` | Read | Yes | — |
| `DEVICE_BUFFER_FRAME_SIZE` | `u32` | Read/Write | Yes | — |
| `DEVICE_BUFFER_FRAME_SIZE_RANGE` | `BufferFrameSizeRange` | Read | No | — |
| `DEVICE_USES_VARIABLE_BUFFER_FRAME_SIZES` | `u32` | Read | No | — |
| `DEVICE_INPUT_LATENCY` | `u32` | Read | No | — |
| `DEVICE_OUTPUT_LATENCY` | `u32` | Read | No | — |
| `DEVICE_SAFETY_OFFSET` | `u32` | Read | No | — |
| `DEVICE_CLOCK_DOMAIN` | `u32` | Read | No | — |
| `DEVICE_HOG_MODE` | `HogMode` | Read/Write | Yes | — |
| `DEVICE_RELATED_DEVICES` | `Vec<u32>` | Read | No | — |
| `DEVICE_PREFERRED_CHANNELS_FOR_STEREO` | `ChannelPair` | Read/Write | No | — |
| `DEVICE_PROCESSOR_OVERLOAD` | `u32` | Read | Yes | — |
| `DEVICE_IO_STOPPED_ABNORMALLY` | `u32` | Read | Yes | — |
| `DEVICE_IO_CYCLE_USAGE` | `f32` | Read/Write | No | — |
| `DEVICE_CLOCK_SOURCE` | `u32` | Read/Write | Yes | — |
| `DEVICE_CLOCK_SOURCES` | `Vec<u32>` | Read | No | — |
| `DEVICE_CLOCK_SOURCE_NAME` | `String` | Read | No | qualifier: `u32` |
| `DEVICE_PLAY_THRU_DESTINATION` | `u32` | Read/Write | Yes | — |
| `DEVICE_PLAY_THRU_DESTINATIONS` | `Vec<u32>` | Read | No | — |
| `DEVICE_PLAY_THRU_DESTINATION_NAME` | `String` | Read | No | qualifier: `u32` |
| `DEVICE_VOLUME_SCALAR` | `f32` | Read/Write | Yes | element |
| `DEVICE_VOLUME_DECIBELS` | `f32` | Read/Write | Yes | element |
| `DEVICE_VOLUME_RANGE_DECIBELS` | `DBRange` | Read | No | element |
| `DEVICE_VOLUME_SCALAR_TO_DECIBELS` | `f32` | Read | No | element |
| `DEVICE_VOLUME_DECIBELS_TO_SCALAR` | `f32` | Read | No | element |
| `DEVICE_SUB_VOLUME_SCALAR` | `f32` | Read/Write | Yes | element |
| `DEVICE_SUB_VOLUME_DECIBELS` | `f32` | Read/Write | Yes | element |
| `DEVICE_SUB_VOLUME_RANGE_DECIBELS` | `DBRange` | Read | No | element |
| `DEVICE_SUB_VOLUME_SCALAR_TO_DECIBELS` | `f32` | Read | No | element |
| `DEVICE_SUB_VOLUME_DECIBELS_TO_SCALAR` | `f32` | Read | No | element |
| `DEVICE_STEREO_PAN` | `f32` | Read/Write | Yes | element |
| `DEVICE_STEREO_PAN_CHANNELS` | `ChannelPair` | Read | No | element |
| `DEVICE_MUTE` | `bool` | Read/Write | Yes | element |
| `DEVICE_SUB_MUTE` | `bool` | Read/Write | Yes | element |
| `DEVICE_SOLO` | `bool` | Read/Write | Yes | element |
| `DEVICE_PHANTOM_POWER` | `bool` | Read/Write | Yes | element |
| `DEVICE_PHASE_INVERT` | `bool` | Read/Write | Yes | element |
| `DEVICE_CLIP_LIGHT` | `bool` | Read/Write | Yes | element |
| `DEVICE_TALKBACK` | `bool` | Read/Write | Yes | element |
| `DEVICE_LISTENBACK` | `bool` | Read/Write | Yes | element |
| `DEVICE_JACK_IS_CONNECTED` | `bool` | Read | Yes | element |
| `DEVICE_DATA_SOURCE` | `u32` | Read/Write | Yes | element |
| `DEVICE_DATA_SOURCES` | `Vec<u32>` | Read | No | element |
| `DEVICE_DATA_SOURCE_NAME` | `String` | Read | No | element + qualifier: `u32` |
| `DEVICE_CHANNEL_NOMINAL_LINE_LEVEL` | `u32` | Read/Write | Yes | element |
| `DEVICE_CHANNEL_NOMINAL_LINE_LEVELS` | `Vec<u32>` | Read | No | element |
| `DEVICE_CHANNEL_NOMINAL_LINE_LEVEL_NAME` | `String` | Read | No | element + qualifier: `u32` |
| `DEVICE_HIGH_PASS_FILTER_SETTING` | `u32` | Read/Write | Yes | element |
| `DEVICE_HIGH_PASS_FILTER_SETTINGS` | `Vec<u32>` | Read | No | element |
| `DEVICE_HIGH_PASS_FILTER_SETTING_NAME` | `String` | Read | No | element + qualifier: `u32` |

### Stream properties

| Constant | Type | Access | Listenable | Extra |
|---|---|---|---|---|
| `STREAM_NAME` | `String` | Read | Yes | — |
| `STREAM_IS_ACTIVE` | `bool` | Read | Yes | — |
| `STREAM_DIRECTION` | `Scope` | Read | No | — |
| `STREAM_LATENCY` | `u32` | Read | Yes | — |
| `STREAM_VIRTUAL_FORMAT` | `StreamDescription` | Read/Write | Yes | — |
| `STREAM_PHYSICAL_FORMAT` | `StreamDescription` | Read/Write | Yes | — |
| `STREAM_AVAILABLE_VIRTUAL_FORMATS` | `Vec<StreamRangedDescription>` | Read | Yes | — |
| `STREAM_AVAILABLE_PHYSICAL_FORMATS` | `Vec<StreamRangedDescription>` | Read | Yes | — |
| `TERMINAL_TYPE` | `TerminalType` | Read | No | — |
| `STARTING_CHANNEL` | `u32` | Read | No | — |

### System properties

| Constant | Type | Access | Listenable | Extra |
|---|---|---|---|---|
| `SYSTEM_NAME` | `String` | Read | No | — |
| `SYSTEM_IS_INITING_OR_EXITING` | `bool` | Read | No | — |
| `SYSTEM_SLEEPING_IS_ALLOWED` | `bool` | Read/Write | Yes | — |
| `SYSTEM_UNLOADING_IS_ALLOWED` | `bool` | Read/Write | No | — |
| `SYSTEM_HOG_MODE_IS_ALLOWED` | `bool` | Read/Write | No | — |
| `SYSTEM_MIX_STEREO_TO_MONO` | `bool` | Read/Write | No | — |
| `SYSTEM_POWER_HINT` | `PowerHint` | Read/Write | No | — |
| `SYSTEM_PROCESS_IS_AUDIBLE` | `bool` | Read/Write | Yes | — |
| `SYSTEM_PROCESS_IS_MASTER` | `bool` | Read | No | — |
| `SYSTEM_USER_SESSION_IS_ACTIVE_OR_HEADLESS` | `bool` | Read | Yes | — |
| `SYSTEM_USER_ID_CHANGED` | `u32` | Read/Write | Yes | — |
| `SYSTEM_SERVICE_RESTARTED` | `u32` | Read | Yes | — |
| `SYSTEM_DEFAULT_SYSTEM_OUTPUT` | `u32` | Read/Write | Yes | — |
| `SYSTEM_BOX_LIST` | `Vec<u32>` | Read | Yes | — |
| `SYSTEM_CLOCK_DEVICE_LIST` | `Vec<u32>` | Read | Yes | — |
| `SYSTEM_PLUGIN_LIST` | `Vec<u32>` | Read | Yes | — |
| `SYSTEM_TAP_LIST` | `Vec<u32>` | Read | Yes | — |
| `SYSTEM_TRANSPORT_MANAGER_LIST` | `Vec<u32>` | Read | No | — |
| `SYSTEM_TRANSLATE_UID_TO_DEVICE` | `u32` | Read | No | qualifier: `String` |
| `SYSTEM_TRANSLATE_UID_TO_BOX` | `u32` | Read | No | qualifier: `String` |
| `SYSTEM_TRANSLATE_UID_TO_CLOCK_DEVICE` | `u32` | Read | No | qualifier: `String` |
| `SYSTEM_TRANSLATE_BUNDLE_ID_TO_PLUGIN` | `u32` | Read | No | qualifier: `String` |
| `SYSTEM_TRANSLATE_BUNDLE_ID_TO_TRANSPORT_MANAGER` | `u32` | Read | No | qualifier: `String` |

## Error handling

All fallible operations return `Result<T, CoreAudioError>`. The error type wraps a typed `ErrorKind` enum and the raw `OSStatus` code. You can match on the kind or inspect the four-character code string:

```rust
use coreaudio::ErrorKind;

match device.get_property(DEVICE_NAME) {
    Ok(name) => println!("{name}"),
    Err(e) => match e.kind() {
        ErrorKind::BadDevice => println!("Invalid device"),
        ErrorKind::Permissions => println!("Device is hogged by another process"),
        _ => println!("Error: {} ('{}')", e, e.stringify_code()),
    }
}
```

## Supported audio formats

The `FormatId` enum covers Linear PCM, AAC (Standard, HE, HEv2, LD, ELD, ELDv2, ELD+SBR, Spatial), Apple Lossless, AC-3, Enhanced AC-3, APAC, AES3, A-Law, AMR, AMR-WB, Opus, and MP3. Unrecognised format IDs are preserved as `FormatId::Unknown(u32)`.

## Roadmap

- Dedicated `AudioObject<Clock>`,  `AudioObject<Box>`, and `AudioObject<Tap>` with unique methods
- Add new wrappers and 'multi-properties' that combine multiple properties into one for things that shouldn't have to be seperate calls
- Properties for device sample format

## Breaking changes - v0.2.1
- Replaced all methods on `SampleRateRange` with methods
    - `.valid_rates()`
    - `.supported_48_khz_rates()`
    - `.supported_44_1_khz_rates()`

## Disclaimer
Apple's documentation on what properties can be listened to is pretty much non existant.
Because of this, almost all writable properties have been made listenable and it will return an error if it turns out not to be.

If you know of any documentation or if a specific property is incorrectly set, please make an issue in the repository and I will fix it at my earliest convenience.

## License

See LICENSE file for details.