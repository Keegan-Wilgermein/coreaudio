//! Property change listeners backed by an MPSC channel.
//!
//! A [`PropertyListener`] registers a CoreAudio property listener callback that
//! reads the new property value and sends it over an internal channel. Callers
//! can poll non-blocking with [`latest`](PropertyListener::latest), drain all
//! pending events with [`all_since_last_check`](PropertyListener::all_since_last_check),
//! or block until a change arrives with [`block_until_change`](PropertyListener::block_until_change).
//! Dropping a `PropertyListener` automatically removes the CoreAudio listener.

#![allow(unsafe_code)]

// ---- Imports --------
use std::{ffi::c_void, marker::PhantomData, sync::mpsc::{self, Receiver, RecvTimeoutError, Sender}, time::Duration};
use coreaudio_sys::{AudioObjectAddPropertyListener, AudioObjectID, AudioObjectPropertyAddress, AudioObjectRemovePropertyListener, OSStatus};
use crate::{Property, errors::{CoreAudioError, ErrorKind, OSStatusCheck}, object::get_property_internal, property::{Listenable, NoExtra}};

// ---- Structs ------------

/// Data shared between the listener registration site and the C callback.
///
/// Boxed and passed through the `*mut c_void` client-data pointer registered
/// with CoreAudio. The callback recovers this by casting back from the pointer.
struct ClientCallbackData<T> {
    /// Function that deserialises raw bytes into a value of type `T`.
    read: fn(&[u8]) -> Result<T, CoreAudioError>,
    /// Sending end of the MPSC channel; the callback sends new values here.
    sender: Sender<T>,
}

impl<T> ClientCallbackData<T> {
    /// Boxes `self` and returns a raw `*mut c_void` suitable for passing to
    /// CoreAudio as client data.
    fn as_c_void(&self) -> *mut c_void {
        Box::into_raw(Box::from(self)) as *mut c_void
    }
}

/// Watches a CoreAudio property for changes and delivers new values over a channel.
///
/// The type parameters carry compile-time information about the property:
/// - `T` — the Rust value type of the property.
/// - `D` — the object kind the property belongs to (`Device`, `Stream`, etc.).
/// - `A` — the access mode (`ReadOnly` or `ReadWrite`).
///
/// Obtain a `PropertyListener` by calling `add_listener` on an [`AudioObject`](crate::AudioObject).
pub struct PropertyListener<T, D, A> {
    /// `AudioObjectID` this listener is registered on.
    id: AudioObjectID,
    /// Property address registered with CoreAudio.
    address: AudioObjectPropertyAddress,
    /// Shared callback context; kept alive for the duration of the listener.
    callback_client_data: ClientCallbackData<T>,
    /// Receiving end of the MPSC channel fed by the CoreAudio callback.
    receiver: Receiver<T>,
    _device: PhantomData<D>,
    _access: PhantomData<A>,
}

impl<T, D, A> Drop for PropertyListener<T, D, A> {
    fn drop(&mut self) {
        unsafe {
            AudioObjectRemovePropertyListener(
                self.id,
                &self.address,
                Some(io_callback::<T, D, A>),
                self.callback_client_data.as_c_void(),
            );
        }
    }
}

impl<T, D, A> PropertyListener<T, D, A> {
    /// Registers a property listener on the object identified by `id`.
    ///
    /// `read` is the deserialisation function taken from the `Property` constant
    /// and is called inside the CoreAudio callback each time the property changes.
    pub(crate) fn try_new(
        id: AudioObjectID,
        address: AudioObjectPropertyAddress,
        read: fn(&[u8]) -> Result<T, CoreAudioError>,
    ) -> Result<Self, CoreAudioError> {
        let (sender, receiver) = mpsc::channel::<T>();

        let callback_client_data = ClientCallbackData {
            read,
            sender,
        };

        let callback_data_ptr = callback_client_data.as_c_void();

        unsafe {
            AudioObjectAddPropertyListener(
                id,
                &address,
                Some(io_callback::<T, D, A>),
                callback_data_ptr,
            ).check()?;
        };

        Ok(
            Self {
                id,
                address,
                callback_client_data,
                receiver,
                _device: PhantomData,
                _access: PhantomData,
            }
        )
    }

    /// Unregisters the listener and releases all associated resources.
    ///
    /// Equivalent to dropping `self`; provided for explicit, readable teardown.
    pub fn remove(self) {
        drop(self);
    }

    /// Returns the most recent property value received since the last call,
    /// or `None` if no change has occurred.
    ///
    /// Does not block. Discards all but the last queued value.
    pub fn latest(&self) -> Option<T> {
        self.receiver.try_iter().last()
    }

    /// Drains and returns all property values received since the last call.
    ///
    /// Does not block. Returns an empty `Vec` if no changes have occurred.
    pub fn all_since_last_check(&self) -> Vec<T> {
        self.receiver.try_iter().collect()
    }

    /// Blocks the calling thread until the property changes, then returns the
    /// new value.
    ///
    /// Returns [`ErrorKind::ListenerHangUp`] if the internal channel is closed.
    pub fn block_until_change(&self) -> Result<T, CoreAudioError> {
        match self.receiver.recv() {
            Ok(value) => Ok(value),
            Err(_) => Err(CoreAudioError::from_error_kind(ErrorKind::ListenerHangUp))
        }
    }

    /// Blocks until a property change arrives or `duration` elapses.
    ///
    /// Returns [`ErrorKind::ListenerTimeOut`] on timeout and
    /// [`ErrorKind::ListenerHangUp`] if the internal channel is closed.
    pub fn block_for_duration(&self, duration: Duration) -> Result<T, CoreAudioError> {
        match self.receiver.recv_timeout(duration) {
            Ok(value) => Ok(value),
            Err(error) => {
                match error {
                    RecvTimeoutError::Timeout => Err(
                        CoreAudioError::from_error_kind(ErrorKind::ListenerTimeout)
                    ),
                    RecvTimeoutError::Disconnected => Err(
                        CoreAudioError::from_error_kind(ErrorKind::ListenerHangUp)
                    ),
                }
            }
        }
    }
}

// ---- Functions ------------

/// CoreAudio property listener callback.
///
/// Called by CoreAudio on a private thread whenever the watched property
/// changes. Reads the new property value and sends it over the MPSC channel
/// so that the owning [`PropertyListener`] can deliver it to the caller.
unsafe extern "C" fn io_callback<T, D, A>(
    device_id: u32,
    _queue: u32,
    address: *const AudioObjectPropertyAddress,
    client_data: *mut c_void,
) -> OSStatus {
    let client_data = unsafe {
        &*(client_data as *mut ClientCallbackData<T>)
    };

    let property: Property<T, D, A, Listenable, NoExtra> = Property::new(
        unsafe { *address },
        client_data.read,
        None,
    );

    let data = match get_property_internal(device_id, property) {
        Ok(data) => data,
        Err(error) => return error.code(),
    };

    match client_data.sender.send(data) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}
