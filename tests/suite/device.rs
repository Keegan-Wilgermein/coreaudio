//! # Device
//!
//! Device properties the test device implements, on the global, input and
//! output scopes
//!
//! Writes are always put back to the value they started at

use super::{CHANGE_TIMEOUT, stream_channels};
use coreaudio::{
    AudioObject, CanListen, ChannelLabel, ChannelLayout, ChannelPair, DEVICE_ACTUAL_SAMPLE_RATE,
    DEVICE_AVAILABLE_SAMPLE_RATES, DEVICE_BUFFER_FRAME_SIZE, DEVICE_BUFFER_FRAME_SIZE_RANGE,
    DEVICE_CAN_BE_DEFAULT, DEVICE_CAN_BE_DEFAULT_SYSTEM, DEVICE_CLOCK_DEVICE, DEVICE_CLOCK_DOMAIN,
    DEVICE_CONFIGURATION_APPLICATION, DEVICE_CONTROL_LIST, DEVICE_INPUT_CAN_BE_DEFAULT,
    DEVICE_INPUT_ELEMENT_NAME, DEVICE_INPUT_LATENCY, DEVICE_INPUT_PREFERRED_CHANNELS_FOR_STEREO,
    DEVICE_INPUT_STREAM_CONFIGURATION, DEVICE_IO_CYCLE_USAGE, DEVICE_IS_ALIVE, DEVICE_IS_HIDDEN,
    DEVICE_IS_RUNNING, DEVICE_IS_RUNNING_SOMEWHERE, DEVICE_NAME, DEVICE_NOMINAL_SAMPLE_RATE,
    DEVICE_OUTPUT_CAN_BE_DEFAULT, DEVICE_OUTPUT_ELEMENT_NAME, DEVICE_OUTPUT_LATENCY,
    DEVICE_OUTPUT_PREFERRED_CHANNELS_FOR_STEREO, DEVICE_OUTPUT_STREAM_CONFIGURATION,
    DEVICE_PREFERRED_CHANNEL_LAYOUT_INPUT, DEVICE_PREFERRED_CHANNEL_LAYOUT_OUTPUT,
    DEVICE_PROCESSOR_OVERLOAD, DEVICE_RELATED_DEVICES, DEVICE_SAFETY_OFFSET_INPUT,
    DEVICE_SAFETY_OFFSET_OUTPUT, DEVICE_TRANSPORT_TYPE, DEVICE_UID,
    DEVICE_VOICE_ACTIVITY_DETECTION_ENABLE, DEVICE_VOICE_ACTIVITY_DETECTION_STATE,
    DEVICE_ZERO_TIME_STAMP_PERIOD, Device, MissingElement, MissingQualifier, NoExtra, Property,
    ReadWrite, SYSTEM_TRANSLATE_UID_TO_DEVICE, Scope, System, TransportType,
};

/// Runs every device test
pub fn run(output: AudioObject<Device>, input: AudioObject<Device>) {
    identity(output);
    state(output);
    sample_rates(output);
    buffer_size(output);
    io_cycle_usage(output);
    latency(output, input);
    defaults(output, input);
    stream_configuration(output, input);
    preferred_stereo(output, input);
    preferred_layout(output, input);
    element_names(output, input);
    voice_activity_detection(input);
}

fn identity(device: AudioObject<Device>) {
    assert_eq!(
        device.get_property(DEVICE_NAME).unwrap(),
        crate::TEST_DEVICE_NAME
    );
    assert!(matches!(
        device.get_property(DEVICE_TRANSPORT_TYPE).unwrap(),
        TransportType::Aggregate
    ));

    // The UID has to lead back to this device
    let uid = device.get_property(DEVICE_UID).unwrap();
    assert!(!uid.is_empty());
    let system = AudioObject::<System>::default();
    assert_eq!(
        system
            .get_property(SYSTEM_TRANSLATE_UID_TO_DEVICE.with_qualifier(uid))
            .unwrap(),
        device.id()
    );

    // Aggregates are configured in Audio MIDI Setup
    assert_eq!(
        device
            .get_property(DEVICE_CONFIGURATION_APPLICATION)
            .unwrap(),
        "com.apple.audio.AudioMIDISetup"
    );

    // An aggregate is related to itself, and has no clock domain or clock
    // device of its own
    assert!(
        device
            .get_property(DEVICE_RELATED_DEVICES)
            .unwrap()
            .contains(&device.id())
    );
    assert_eq!(device.get_property(DEVICE_CLOCK_DOMAIN).unwrap(), 0);
    device.get_property(DEVICE_CLOCK_DEVICE).unwrap();
    device.get_property(DEVICE_ZERO_TIME_STAMP_PERIOD).unwrap();

    // Aggregates have no controls of their own
    assert!(device.get_property(DEVICE_CONTROL_LIST).unwrap().is_empty());
}

fn state(device: AudioObject<Device>) {
    assert!(device.get_property(DEVICE_IS_ALIVE).unwrap());
    assert!(!device.get_property(DEVICE_IS_HIDDEN).unwrap());

    // Running here implies running somewhere
    let running = device.get_property(DEVICE_IS_RUNNING).unwrap();
    let running_somewhere = device.get_property(DEVICE_IS_RUNNING_SOMEWHERE).unwrap();
    assert!(!running || running_somewhere);

    // A notification property, but it still reads
    device.get_property(DEVICE_PROCESSOR_OVERLOAD).unwrap();
}

/// Switches to another supported rate, waits for it to take effect, then
/// switches back
fn sample_rates(device: AudioObject<Device>) {
    let original = device.get_property(DEVICE_NOMINAL_SAMPLE_RATE).unwrap();
    let available = device.get_property(DEVICE_AVAILABLE_SAMPLE_RATES).unwrap();

    let supported = |rate: f64| {
        available
            .iter()
            .any(|range| range.get_min() <= rate && rate <= range.get_max())
    };
    assert!(supported(original), "{original} Hz isn't in {available:?}");

    // Not running, so the measured rate is just the nominal one
    if !device.get_property(DEVICE_IS_RUNNING).unwrap() {
        assert_eq!(
            device.get_property(DEVICE_ACTUAL_SAMPLE_RATE).unwrap(),
            original
        );
    }

    let other = [48000.0, 44100.0]
        .into_iter()
        .find(|rate| *rate != original && supported(*rate))
        .expect("the test device should support 44.1 or 48 kHz");

    set_and_wait(device, || DEVICE_NOMINAL_SAMPLE_RATE, other);
    set_and_wait(device, || DEVICE_NOMINAL_SAMPLE_RATE, original);
}

/// The buffer size is per process, so changing it doesn't disturb anything
/// else using the device
fn buffer_size(device: AudioObject<Device>) {
    let original = device.get_property(DEVICE_BUFFER_FRAME_SIZE).unwrap();
    let valid = device
        .get_property(DEVICE_BUFFER_FRAME_SIZE_RANGE)
        .unwrap()
        .valid_sizes();
    assert!(!valid.is_empty());

    let other = *valid.iter().find(|size| **size != original).unwrap();
    device
        .set_property(DEVICE_BUFFER_FRAME_SIZE, other)
        .unwrap();
    assert_eq!(
        device.get_property(DEVICE_BUFFER_FRAME_SIZE).unwrap(),
        other
    );

    device
        .set_property(DEVICE_BUFFER_FRAME_SIZE, original)
        .unwrap();
    assert_eq!(
        device.get_property(DEVICE_BUFFER_FRAME_SIZE).unwrap(),
        original
    );
}

/// Also per process
fn io_cycle_usage(device: AudioObject<Device>) {
    let original = device.get_property(DEVICE_IO_CYCLE_USAGE).unwrap();
    assert!((0.0..=1.0).contains(&original));

    device.set_property(DEVICE_IO_CYCLE_USAGE, 0.5).unwrap();
    assert_eq!(device.get_property(DEVICE_IO_CYCLE_USAGE).unwrap(), 0.5);

    device
        .set_property(DEVICE_IO_CYCLE_USAGE, original)
        .unwrap();
    assert_eq!(
        device.get_property(DEVICE_IO_CYCLE_USAGE).unwrap(),
        original
    );
}

fn latency(output: AudioObject<Device>, input: AudioObject<Device>) {
    output.get_property(DEVICE_OUTPUT_LATENCY).unwrap();
    output.get_property(DEVICE_SAFETY_OFFSET_OUTPUT).unwrap();
    input.get_property(DEVICE_INPUT_LATENCY).unwrap();
    input.get_property(DEVICE_SAFETY_OFFSET_INPUT).unwrap();
}

/// The test device has both sides, so it can be the default on either
fn defaults(output: AudioObject<Device>, input: AudioObject<Device>) {
    output.get_property(DEVICE_CAN_BE_DEFAULT).unwrap();
    output.get_property(DEVICE_CAN_BE_DEFAULT_SYSTEM).unwrap();
    assert!(output.get_property(DEVICE_OUTPUT_CAN_BE_DEFAULT).unwrap());
    assert!(input.get_property(DEVICE_INPUT_CAN_BE_DEFAULT).unwrap());
}

/// The stream configuration has to match the streams themselves
fn stream_configuration(output: AudioObject<Device>, input: AudioObject<Device>) {
    let outputs = output
        .get_property(DEVICE_OUTPUT_STREAM_CONFIGURATION)
        .unwrap();
    assert!(!outputs.is_empty());
    assert_eq!(outputs, stream_channels(output, Scope::Output));

    let inputs = input
        .get_property(DEVICE_INPUT_STREAM_CONFIGURATION)
        .unwrap();
    assert!(!inputs.is_empty());
    assert_eq!(inputs, stream_channels(input, Scope::Input));
}

fn preferred_stereo(output: AudioObject<Device>, input: AudioObject<Device>) {
    swap_preferred_stereo(output, Scope::Output, || {
        DEVICE_OUTPUT_PREFERRED_CHANNELS_FOR_STEREO
    });
    swap_preferred_stereo(input, Scope::Input, || {
        DEVICE_INPUT_PREFERRED_CHANNELS_FOR_STEREO
    });
}

/// Swaps the preferred stereo pair on one side, then puts it back
fn swap_preferred_stereo<L>(
    device: AudioObject<Device>,
    scope: Scope,
    property: impl Fn() -> Property<ChannelPair, Device, ReadWrite, L, NoExtra>,
) {
    let channels: u32 = stream_channels(device, scope).iter().sum();
    let original = device.get_property(property()).unwrap();
    assert!((1..=channels).contains(&original.left()));
    assert!((1..=channels).contains(&original.right()));

    let swapped = ChannelPair::from([original.right(), original.left()]);
    device.set_property(property(), swapped).unwrap();
    assert_eq!(
        device.get_property(property()).unwrap().as_array(),
        swapped.as_array()
    );

    device.set_property(property(), original).unwrap();
    assert_eq!(
        device.get_property(property()).unwrap().as_array(),
        original.as_array()
    );
}

fn preferred_layout(output: AudioObject<Device>, input: AudioObject<Device>) {
    stereo_layout(output, Scope::Output, || {
        DEVICE_PREFERRED_CHANNEL_LAYOUT_OUTPUT
    });
    stereo_layout(input, Scope::Input, || {
        DEVICE_PREFERRED_CHANNEL_LAYOUT_INPUT
    });
}

/// Gives the first two channels on one side a stereo layout, then puts the
/// original layout back
fn stereo_layout<L>(
    device: AudioObject<Device>,
    scope: Scope,
    property: impl Fn() -> Property<ChannelLayout, Device, ReadWrite, L, NoExtra>,
) {
    let channels: u32 = stream_channels(device, scope).iter().sum();
    let original = device.get_property(property()).unwrap();
    assert_eq!(original.descriptions().len(), channels as usize);

    let mut labels = vec![ChannelLabel::UNUSED; channels as usize];
    labels[0] = ChannelLabel::LEFT;
    labels[1] = ChannelLabel::RIGHT;

    device
        .set_property(property(), ChannelLayout::from(labels.clone()))
        .unwrap();
    assert_eq!(device.get_property(property()).unwrap().labels(), labels);

    device.set_property(property(), original.clone()).unwrap();
    assert_eq!(device.get_property(property()).unwrap(), original);
}

/// Every channel has a name on each side, even if it's empty
fn element_names(output: AudioObject<Device>, input: AudioObject<Device>) {
    let outputs: u32 = stream_channels(output, Scope::Output).iter().sum();
    for channel in 0..=outputs {
        output
            .get_property(DEVICE_OUTPUT_ELEMENT_NAME.for_element(channel))
            .unwrap();
    }

    let inputs: u32 = stream_channels(input, Scope::Input).iter().sum();
    for channel in 0..=inputs {
        input
            .get_property(DEVICE_INPUT_ELEMENT_NAME.for_element(channel))
            .unwrap();
    }
}

/// Input isn't running, so no voice can be detected whether detection is on
/// or not
fn voice_activity_detection(input: AudioObject<Device>) {
    input
        .get_property(DEVICE_VOICE_ACTIVITY_DETECTION_ENABLE)
        .unwrap();

    if !input.get_property(DEVICE_IS_RUNNING_SOMEWHERE).unwrap() {
        assert!(
            !input
                .get_property(DEVICE_VOICE_ACTIVITY_DETECTION_STATE)
                .unwrap()
        );
    }
}

/// Writes `value` and waits for CoreAudio to report it, for properties that
/// change asynchronously
fn set_and_wait<L: CanListen>(
    device: AudioObject<Device>,
    property: impl Fn() -> Property<f64, Device, ReadWrite, L, NoExtra>,
    value: f64,
) {
    if device.get_property(property()).unwrap() == value {
        return;
    }

    let listener = device.add_listener(property()).unwrap();
    device.set_property(property(), value).unwrap();

    while device.get_property(property()).unwrap() != value {
        listener.block_for_duration(CHANGE_TIMEOUT).unwrap();
    }
}
