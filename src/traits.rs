//! # Traits

// ---- Imports ------------
use crate::{Device, Global, Property, Stream, System, property::{Listenable, NeedBoth, NeedElement, NeedQualifier, NoExtra, ReadWrite, encode_u32}};

// ---- Traits ------------
#[diagnostic::on_unimplemented(
    message = "`{Self}` properties are incompatible with `AudioObject<{T}>`",
    label = "Use this with `AudioObject<{Self}>` instead",
)]
/// Checks property compatibility with `AudioObject<T>`
pub trait ObjectCompatibleWith<T> {}

impl ObjectCompatibleWith<System> for System {}
impl ObjectCompatibleWith<Device> for Device {}
impl ObjectCompatibleWith<Stream> for Stream {}

impl ObjectCompatibleWith<System> for Global {}
impl ObjectCompatibleWith<Device> for Global {}
impl ObjectCompatibleWith<Stream> for Global {}

#[diagnostic::on_unimplemented(
    message = "this property is read-only",
    label = "`set_property()` requires properties with `ReadWrite`",
)]
/// Checks property compatibility with `ReadWrite`
pub(crate) trait Writeable {}

impl Writeable for ReadWrite {}

#[diagnostic::on_unimplemented(
    message = "this property does not support listeners",
    label = "`add_listener()` requires propertes with `Listenable`",
)]
/// Checks property compatibility with `Listenable`
pub(crate) trait CanListen {}

impl CanListen for Listenable {}

#[diagnostic::on_unimplemented(
    message = "this property requires additional data before it can be used",
    label = "check for `.with_qualifier()` or `.for_element()` methods on this property",
)]

/// Checks for no extra data on a property
pub(crate) trait HasAllData {}

impl HasAllData for NoExtra {}

/// Method to add qualifier data to property
pub(crate) trait IntoQualifierBytes {
    fn into_bytes(self) -> Vec<u8>;
}

impl IntoQualifierBytes for u32 {
    fn into_bytes(self) -> Vec<u8> {
        encode_u32(self)
    }
}

#[diagnostic::on_unimplemented(
    message = "this property does not accept qualifier data",
)]
/// Method to add qualifier data to property
pub(crate) trait MissingQualifier<T> {
    type Output;

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

#[diagnostic::on_unimplemented(
    message = "this property does not accept element data",
)]
/// Method to add element data to property
pub(crate) trait MissingElement {
    type Output;

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
