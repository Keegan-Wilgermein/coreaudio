//! Marker traits and builder traits for compile-time property validation.
//!
//! These traits are used as bounds on `get_property`, `set_property`, and
//! `add_listener` to reject invalid calls at compile time with actionable
//! error messages. They also provide the builder methods (`.with_qualifier()`,
//! `.for_element()`) that complete partially-specified property constants before
//! they can be used.

// ---- Imports ------------
use crate::{Device, Global, Property, Stream, System, property::{Listenable, NeedBoth, NeedElement, NeedQualifier, NoExtra, ReadWrite, encode_string, encode_u32, encode_vec_u32}};

// ---- Traits ------------

/// Asserts that a property's object type is compatible with an `AudioObject<T>`.
///
/// Implemented for the pairings that CoreAudio permits:
/// - `System` and `Global` properties work with `AudioObject<System>`.
/// - `Device` and `Global` properties work with `AudioObject<Device>`.
/// - `Stream` and `Global` properties work with `AudioObject<Stream>`.
#[diagnostic::on_unimplemented(
    message = "`{Self}` properties are incompatible with `AudioObject<{T}>`",
    label = "Use this with `AudioObject<{Self}>` instead",
)]
pub trait ObjectCompatibleWith<T> {}

impl ObjectCompatibleWith<System> for System {}
impl ObjectCompatibleWith<Device> for Device {}
impl ObjectCompatibleWith<Stream> for Stream {}

impl ObjectCompatibleWith<System> for Global {}
impl ObjectCompatibleWith<Device> for Global {}
impl ObjectCompatibleWith<Stream> for Global {}

/// Asserts that a property's access mode permits writing.
///
/// Only `ReadWrite` properties implement this trait; attempting to call
/// `set_property` with a `ReadOnly` property produces a compile error.
#[diagnostic::on_unimplemented(
    message = "this property is read-only",
    label = "`set_property()` requires properties with `ReadWrite`",
)]
pub trait Writeable {}

impl Writeable for ReadWrite {}

/// Asserts that a property supports change listeners.
///
/// Only `Listenable` properties implement this trait; attempting to call
/// `add_listener` with a `Silent` property produces a compile error.
#[diagnostic::on_unimplemented(
    message = "this property does not support listeners",
    label = "`add_listener()` requires propertes with `Listenable`",
)]
pub trait CanListen {}

impl CanListen for Listenable {}

/// Asserts that a property has all required data (element and qualifier) set.
///
/// Properties with an `E` parameter of `NoExtra` are fully specified and can
/// be passed directly to `get_property`, `set_property`, or `add_listener`.
/// Properties with `NeedElement`, `NeedQualifier`, or `NeedBoth` must first be
/// completed via `.for_element()` or `.with_qualifier()`.
#[diagnostic::on_unimplemented(
    message = "this property requires additional data before it can be used",
    label = "check for `.with_qualifier()` or `.for_element()` methods on this property",
)]
pub trait HasAllData {}

impl HasAllData for NoExtra {}

/// Converts a qualifier value into raw bytes for a property address.
///
/// Implement this for any type that can serve as qualifier data. The built-in
/// implementations cover the types accepted by CoreAudio qualifiers (`u32`,
/// `Vec<u32>`, and `String`).
#[diagnostic::on_unimplemented(
    message = "incorrect qualifier data type",
    label = "check the `.with_qualifier()` method for expected type",
)]
pub trait IntoQualifierBytes {
    /// Serialises the qualifier value into a byte vector suitable for passing
    /// to CoreAudio.
    fn into_bytes(self) -> Vec<u8>;
}

impl IntoQualifierBytes for u32 {
    fn into_bytes(self) -> Vec<u8> {
        encode_u32(self)
    }
}

impl IntoQualifierBytes for Vec<u32> {
    fn into_bytes(self) -> Vec<u8> {
        encode_vec_u32(self)
    }
}

impl IntoQualifierBytes for String {
    fn into_bytes(self) -> Vec<u8> {
        encode_string(self)
    }
}

/// Attaches qualifier data to a property that requires it.
///
/// Calling `.with_qualifier(value)` on a `NeedQualifier<T>` or `NeedBoth<T>`
/// property transitions the `E` type parameter, removing the qualifier
/// requirement. The resulting property may be fully specified (`NoExtra`) or
/// may still need an element (`NeedElement`).
#[diagnostic::on_unimplemented(
    message = "this property does not accept qualifier data",
)]
pub trait MissingQualifier<T> {
    /// The property type produced after the qualifier is attached.
    type Output;

    /// Attaches `qualifier` to this property and returns the updated property.
    fn with_qualifier(self, qualifier: T) -> Self::Output
    where
        T: IntoQualifierBytes;
}

impl<V, D, A, L, T> MissingQualifier<T> for Property<V, D, A, L, NeedQualifier<T>> {
    type Output = Property<V, D, A, L, NoExtra>;

    fn with_qualifier(self, qualifier: T) -> Self::Output
    where
        T: IntoQualifierBytes,
    {
        let mut new: Property<V, D, A, L, NoExtra> = Property::new(
            self.address,
            self.read,
            self.encode,
        );

        new.qualifier = Some(qualifier.into_bytes());
        new
    }
}

impl<V, D, A, L, T> MissingQualifier<T> for Property<V, D, A, L, NeedBoth<T>> {
    type Output = Property<V, D, A, L, NeedElement>;

    fn with_qualifier(self, qualifier: T) -> Self::Output
    where
        T: IntoQualifierBytes,
    {
        let mut new: Property<V, D, A, L, NeedElement> = Property::new(
            self.address,
            self.read,
            self.encode,
        );

        new.qualifier = Some(qualifier.into_bytes());
        new
    }
}

/// Sets the element on a property that requires one.
///
/// Calling `.for_element(n)` on a `NeedElement` or `NeedBoth<T>` property
/// writes `n` into `mElement` of the property address, transitioning the `E`
/// type parameter to remove the element requirement.
#[diagnostic::on_unimplemented(
    message = "this property does not accept element data",
)]
pub trait MissingElement {
    /// The property type produced after the element is set.
    type Output;

    /// Sets `element` as the `mElement` of the property address and returns
    /// the updated property.
    fn for_element(self, element: u32) -> Self::Output;
}

impl<V, D, A, L> MissingElement for Property<V, D, A, L, NeedElement> {
    type Output = Property<V, D, A, L, NoExtra>;

    fn for_element(self, element: u32) -> Self::Output {
        let mut new: Property<V, D, A, L, NoExtra> = Property::new(
            self.address,
            self.read,
            self.encode,
        );
        new.address.mElement = element;

        new.qualifier = self.qualifier;
        new
    }
}

impl<V, D, A, L, T> MissingElement for Property<V, D, A, L, NeedBoth<T>> {
    type Output = Property<V, D, A, L, NeedQualifier<T>>;

    fn for_element(self, element: u32) -> Self::Output {
        let mut new: Property<V, D, A, L, NeedQualifier<T>> = Property::new(
            self.address,
            self.read,
            self.encode,
        );
        new.address.mElement = element;

        new.qualifier = self.qualifier;
        new
    }
}
