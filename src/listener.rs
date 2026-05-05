//! # Listeners

#![allow(unsafe_code)]

// ---- Imports --------
use std::{ffi::c_void, marker::PhantomData, sync::mpsc::{self, Receiver, RecvTimeoutError, Sender}, time::Duration};
use coreaudio_sys::{AudioObjectAddPropertyListener, AudioObjectID, AudioObjectPropertyAddress, AudioObjectRemovePropertyListener, OSStatus};
use crate::{Property, errors::{CoreAudioError, ErrorKind, OSStatusCheck}, object::get_property_internal, property::{Listenable, NoExtra}};

// ---- Structs ------------
struct ClientCallbackData<T> {
    read: fn(&[u8]) -> Result<T, CoreAudioError>,
    sender: Sender<T>,
}

impl<T> ClientCallbackData<T> {
    fn as_c_void(&self) -> *mut c_void {
        Box::into_raw(Box::from(self)) as *mut c_void
    }
}

pub struct PropertyListener<T, D, A> {
    id: AudioObjectID,
    address: AudioObjectPropertyAddress,
    callback_client_data: ClientCallbackData<T>,
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

    pub fn remove(self) {
        drop(self);
    }

    pub fn latest(&self) -> Option<T> {
        self.receiver.try_iter().last()
    }

    pub fn all_since_last_check(&self) -> Vec<T> {
        self.receiver.try_iter().collect()
    }

    pub fn block_until_change(&self) -> Result<T, CoreAudioError> {
        match self.receiver.recv() {
            Ok(value) => Ok(value),
            Err(_) => Err(CoreAudioError::from_error_kind(ErrorKind::ListenerHangUp))
        }
    }

    pub fn block_for_duration(&self, duration: Duration) -> Result<T, CoreAudioError> {
        match self.receiver.recv_timeout(duration) {
            Ok(value) => Ok(value),
            Err(error) => {
                match error {
                    RecvTimeoutError::Timeout => Err(
                        CoreAudioError::from_error_kind(ErrorKind::ListenerTimeOut)
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
