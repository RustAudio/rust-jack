//! Properties, AKA [Meta Data](https://jackaudio.org/api/group__Metadata.html)
//!
use std::panic::catch_unwind;

use j::jack_uuid_t as uuid;
use jack_sys as j;

/// A description of a Metadata change describint a creation, change or deletion, its owner
/// `subject` and `key`.
#[derive(Debug, PartialEq, Eq)]
pub enum PropertyChange<'a> {
    Created { subject: uuid, key: &'a str },
    Changed { subject: uuid, key: &'a str },
    Deleted { subject: uuid, key: &'a str },
}

/// A trait for reacting to property changes.
///
/// # Remarks
///
/// * Only used if the `metadata` feature is enabled.
pub trait PropertyChangeHandler: Send {
    fn property_changed(&mut self, change: &PropertyChange);
}

#[allow(dead_code)] //dead if we haven't enabled metadata
pub(crate) unsafe extern "C" fn property_changed<P>(
    subject: j::jack_uuid_t,
    key: *const ::libc::c_char,
    change: j::jack_property_change_t,
    arg: *mut ::libc::c_void,
) where
    P: PropertyChangeHandler,
{
    let res = catch_unwind(|| {
        let h: &mut P = &mut *(arg as *mut P);
        let key_c = std::ffi::CStr::from_ptr(key);
        let key = key_c.to_str().expect("to convert key to valid str");
        let c = match change {
            j::PropertyCreated => PropertyChange::Created { subject, key },
            j::PropertyDeleted => PropertyChange::Deleted { subject, key },
            _ => PropertyChange::Changed { subject, key },
        };
        h.property_changed(&c);
    });
    if let Err(err) = res {
        eprintln!("{err:?}");
        std::mem::forget(err);
    }
}

#[cfg(feature = "metadata")]
pub use metadata::*;

#[cfg(feature = "metadata")]
mod metadata {
    use super::{j, uuid, PropertyChange, PropertyChangeHandler};
    use crate::Error;
    use std::{
        collections::HashMap,
        ffi,
        mem::MaybeUninit,
        ptr::{self, NonNull},
    };

    use crate::Client;

    /// A helper enum, allowing for sending changes between threads.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum PropertyChangeOwned {
        Created { subject: uuid, key: String },
        Changed { subject: uuid, key: String },
        Deleted { subject: uuid, key: String },
    }

    /// A piece of Metadata on a Jack `subject`: either a port or a client.
    ///
    /// See the JACK Metadata API [description](https://jackaudio.org/metadata/) and
    /// [documentation](https://jackaudio.org/api/group__Metadata.html) and for more info.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Property {
        value: String,
        typ: Option<String>,
    }

    /// A map of Metadata `key`s, URI Strings, to `Property`s, value and optional type Strings, for a given subject.
    pub type PropertyMap = HashMap<String, Property>;

    /// Wrap a closure that chan handle a `property_changed` callback.
    /// This is called for every property that changes.
    pub struct ClosurePropertyChangeHandler<F>
    where
        F: 'static + Send + FnMut(&PropertyChange),
    {
        func: F,
    }

    impl<F> ClosurePropertyChangeHandler<F>
    where
        F: 'static + Send + FnMut(&PropertyChange),
    {
        /// Create a new `PropertyChangeHandler` from a closure.
        pub fn new(func: F) -> ClosurePropertyChangeHandler<F> {
            ClosurePropertyChangeHandler { func }
        }
    }

    impl<F> PropertyChangeHandler for ClosurePropertyChangeHandler<F>
    where
        F: 'static + Send + FnMut(&PropertyChange),
    {
        fn property_changed(&mut self, change: &PropertyChange) {
            (self.func)(change)
        }
    }

    // Helper to map 0 return to Ok
    fn map_error<F: FnOnce() -> ::libc::c_int>(func: F) -> Result<(), Error> {
        match func() {
            0 => Ok(()),
            error_code => Err(Error::UnknownError { error_code }),
        }
    }

    //helper to convert to an Option<PropertyMap> and free
    unsafe fn description_to_map_free(
        description: *mut j::jack_description_t,
    ) -> Option<PropertyMap> {
        let description = NonNull::new(description)?;
        let mut properties = HashMap::new();
        let len = description.as_ref().property_cnt;
        // The check is required as from_raw_parts doesn't like receiving a null ptr, even if the
        // length is 0.
        if len > 0 {
            let properties_slice =
                std::slice::from_raw_parts(description.as_ref().properties, len as usize);
            for prop in properties_slice {
                let typ = if prop._type.is_null() {
                    None
                } else {
                    Some(
                        ffi::CStr::from_ptr(prop._type)
                            .to_str()
                            .expect("to convert type to str")
                            .to_string(),
                    )
                };
                properties.insert(
                    ffi::CStr::from_ptr(prop.key)
                        .to_str()
                        .expect("to turn key to str")
                        .to_string(),
                    Property::new(
                        ffi::CStr::from_ptr(prop.data)
                            .to_str()
                            .expect("to turn data to str"),
                        typ,
                    ),
                );
            }
        }
        j::jack_free_description(description.as_ptr(), 0);
        Some(properties)
    }

    impl Property {
        /// Create a property.
        ///
        /// # Arguments
        ///
        /// * `value` - The value of the property.
        /// * `typ` - The optional type of the property. Either a MIME type or URI.
        pub fn new<V: ToString>(value: V, typ: Option<String>) -> Self {
            Self {
                value: value.to_string(),
                typ,
            }
        }

        /// Get the "value" of a property.
        pub fn value(&self) -> &str {
            &self.value
        }

        /// Get the "type" of a property, if it has been set.
        /// Either a MIME type or URI.
        pub fn typ(&self) -> Option<&str> {
            self.typ.as_deref()
        }
    }

    #[cfg(feature = "metadata")]
    impl Client {
        /// Get a property from a subject.
        ///
        /// # Arguments
        ///
        /// * `subject` - The subject of the property.
        /// * `key` - The key of the property, a URI String.
        pub fn property_get(&self, subject: uuid, key: &str) -> Option<Property> {
            let key = ffi::CString::new(key).expect("key to be convert to CString");
            let mut value: MaybeUninit<*mut ::libc::c_char> = MaybeUninit::uninit();
            let mut typ: MaybeUninit<*mut ::libc::c_char> = MaybeUninit::uninit();

            unsafe {
                if j::jack_get_property(subject, key.as_ptr(), value.as_mut_ptr(), typ.as_mut_ptr())
                    == 0
                {
                    let value = value.assume_init();
                    let typ = typ.assume_init();
                    let r = Some(Property::new(
                        ffi::CStr::from_ptr(value).to_str().unwrap(),
                        if typ.is_null() {
                            None
                        } else {
                            Some(ffi::CStr::from_ptr(typ).to_str().unwrap().to_string())
                        },
                    ));
                    j::jack_free(value as _);
                    if !typ.is_null() {
                        j::jack_free(typ as _)
                    }
                    r
                } else {
                    None
                }
            }
        }

        /// Get all the properties from a subject.
        ///
        /// # Arguments
        ///
        /// * `subject` - The subject of the properties.
        ///
        /// # Remarks
        ///
        /// * The Jack API calls this data a 'description'.
        pub fn property_get_subject(&self, subject: uuid) -> Option<PropertyMap> {
            let mut description: MaybeUninit<j::jack_description_t> = MaybeUninit::uninit();
            unsafe {
                let _ = j::jack_get_properties(subject, description.as_mut_ptr());
                description_to_map_free(description.as_mut_ptr())
            }
        }

        /// Get all the properties from all the subjects with Metadata.
        ///
        /// # Remarks
        ///
        /// * The Jack API calls these maps 'descriptions'.
        pub fn property_get_all(&self) -> HashMap<uuid, PropertyMap> {
            let mut map = HashMap::new();
            let mut descriptions: MaybeUninit<*mut j::jack_description_t> = MaybeUninit::uninit();
            unsafe {
                let cnt = j::jack_get_all_properties(descriptions.as_mut_ptr());
                if cnt > 0 {
                    let descriptions = descriptions.assume_init();
                    for des in std::slice::from_raw_parts_mut(descriptions, cnt as usize) {
                        let uuid = (des).subject;
                        if let Some(dmap) = description_to_map_free(des) {
                            map.insert(uuid, dmap);
                        }
                    }
                    j::jack_free(descriptions as _);
                }
            }
            map
        }

        /// Set a property.
        ///
        /// # Arguments
        ///
        /// * `subject` - The subject of the property.
        /// * `key` - The key of the property. A URI string.
        pub fn property_set(
            &self,
            subject: uuid,
            key: &str,
            property: &Property,
        ) -> Result<(), Error> {
            let key = ffi::CString::new(key).expect("to create cstring from key");
            let value =
                ffi::CString::new(property.value.as_str()).expect("to create cstring from value");
            map_error(|| unsafe {
                if let Some(t) = property.typ() {
                    let t = ffi::CString::new(t).unwrap();
                    j::jack_set_property(
                        self.raw(),
                        subject,
                        key.as_ptr(),
                        value.as_ptr(),
                        t.as_ptr(),
                    )
                } else {
                    j::jack_set_property(
                        self.raw(),
                        subject,
                        key.as_ptr(),
                        value.as_ptr(),
                        ptr::null(),
                    )
                }
            })
        }

        /// Remove a single property from a subject.
        ///
        /// # Arguments
        ///
        /// * `subject` - The subject to remove all properties from.
        /// * `key` - The key of the property to be removed. A URI string.
        pub fn property_remove(&self, subject: uuid, key: &str) -> Result<(), Error> {
            let key = ffi::CString::new(key).expect("to create cstring from key");
            map_error(|| unsafe { j::jack_remove_property(self.raw(), subject, key.as_ptr()) })
        }

        /// Remove all properties from a subject.
        ///
        /// # Arguments
        ///
        /// * `subject` - The subject to remove all properties from.
        pub fn property_remove_subject(&self, subject: uuid) -> Result<(), Error> {
            unsafe {
                if j::jack_remove_properties(self.raw(), subject) == -1 {
                    Err(Error::UnknownError { error_code: -1 })
                } else {
                    Ok(())
                }
            }
        }

        /// Remove all properties.
        ///
        /// # Remarks
        ///
        /// * **WARNING!!** This deletes all Metadata managed by a running JACK server.
        pub fn property_remove_all(&self) -> Result<(), Error> {
            map_error(|| unsafe { j::jack_remove_all_properties(self.raw()) })
        }
    }

    impl<'a> From<&PropertyChange<'a>> for PropertyChangeOwned {
        fn from(property: &PropertyChange<'a>) -> Self {
            match property {
                PropertyChange::Created { subject, key } => Self::Created {
                    subject: *subject,
                    key: key.to_string(),
                },
                PropertyChange::Changed { subject, key } => Self::Changed {
                    subject: *subject,
                    key: key.to_string(),
                },
                PropertyChange::Deleted { subject, key } => Self::Deleted {
                    subject: *subject,
                    key: key.to_string(),
                },
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::client::{Client, ClientOptions};
        use std::sync::mpsc::{channel, Sender};

        #[test]
        fn can_set_and_get() {
            let (c, _) = Client::new("dummy", ClientOptions::NO_START_SERVER).unwrap();

            let prop1 = Property::new("foo", None);
            assert_eq!(c.property_set(c.uuid(), "blah", &prop1), Ok(()));

            let prop2 = Property::new(
                "http://churchofrobotron.com/2084",
                Some("robot apocalypse".into()),
            );
            assert_eq!(c.property_set(c.uuid(), "mutant", &prop2), Ok(()));

            assert_eq!(None, c.property_get(c.uuid(), "soda"));
            assert_eq!(Some(prop1.clone()), c.property_get(c.uuid(), "blah"));
            assert_eq!(Some(prop2.clone()), c.property_get(c.uuid(), "mutant"));

            //get subject
            let sub = c.property_get_subject(c.uuid());
            assert!(sub.is_some());
            let sub = sub.unwrap();
            assert_eq!(2, sub.len());

            assert_eq!(sub.get("blah"), Some(&prop1));
            assert_eq!(sub.get("mutant"), Some(&prop2));
            assert_eq!(sub.get("asdf"), None);

            //get all
            let all = c.property_get_all();
            assert_ne!(0, all.len());

            let sub = all.get(&c.uuid());
            assert!(sub.is_some());
            let sub = sub.unwrap();
            assert_eq!(2, sub.len());

            assert_eq!(sub.get("blah"), Some(&prop1));
            assert_eq!(sub.get("mutant"), Some(&prop2));
            assert_eq!(sub.get("asdf"), None);
        }

        #[test]
        fn can_remove() {
            let (c1, _) = Client::new("client1", ClientOptions::NO_START_SERVER).unwrap();
            let (c2, _) = Client::new("client2", ClientOptions::NO_START_SERVER).unwrap();
            let prop1 = Property::new("foo", None);
            let prop2 = Property::new(
                "http://churchofrobotron.com/2084",
                Some("robot apocalypse".into()),
            );

            assert_eq!(c1.property_set(c1.uuid(), "blah", &prop1), Ok(()));
            assert_eq!(c1.property_set(c2.uuid(), "blah", &prop1), Ok(()));
            assert_eq!(c2.property_set(c1.uuid(), "mutant", &prop2), Ok(()));
            assert_eq!(c2.property_set(c2.uuid(), "mutant", &prop2), Ok(()));

            assert_eq!(Some(prop1.clone()), c1.property_get(c1.uuid(), "blah"));
            assert_eq!(Some(prop1.clone()), c1.property_get(c2.uuid(), "blah"));
            assert_eq!(Some(prop2.clone()), c1.property_get(c1.uuid(), "mutant"));
            assert_eq!(Some(prop2.clone()), c1.property_get(c2.uuid(), "mutant"));

            assert_eq!(Ok(()), c1.property_remove(c1.uuid(), "blah"));
            assert_eq!(None, c1.property_get(c1.uuid(), "blah"));

            //with other client
            assert_eq!(Ok(()), c2.property_remove(c1.uuid(), "mutant"));
            assert_eq!(None, c1.property_get(c1.uuid(), "mutant"));

            //second time, error
            assert!(matches!(
                c2.property_remove(c1.uuid(), "mutant"),
                Err(Error::UnknownError { .. })
            ));

            assert_eq!(Some(prop1), c2.property_get(c2.uuid(), "blah"));
            assert_eq!(Some(prop2), c2.property_get(c2.uuid(), "mutant"));

            assert_eq!(Ok(()), c1.property_remove_subject(c2.uuid()));
            assert_eq!(None, c2.property_get(c2.uuid(), "blah"));
            assert_eq!(None, c2.property_get(c2.uuid(), "mutant"));

            //second time, okay
            assert_eq!(Ok(()), c1.property_remove_subject(c2.uuid()));
            assert_eq!(Ok(()), c2.property_remove_subject(c2.uuid()));
            assert_eq!(None, c2.property_get(c2.uuid(), "blah"));
            assert_eq!(None, c2.property_get(c2.uuid(), "mutant"));

            assert_eq!(Ok(()), c2.property_remove_subject(c1.uuid()));
            assert_eq!(Ok(()), c1.property_remove_subject(c1.uuid()));
        }

        #[test]
        fn can_property_remove_all() {
            let (c, _) = Client::new("dummy", ClientOptions::NO_START_SERVER).unwrap();
            let prop = Property::new("foo", Some("bar".into()));
            assert_eq!(c.property_set(c.uuid(), "blah", &prop), Ok(()));

            let sub = c.property_get_subject(c.uuid());
            assert!(sub.is_some());
            let sub = sub.unwrap();
            assert_eq!(1, sub.len());

            let all = c.property_get_all();
            assert_ne!(0, all.len());

            assert_eq!(c.property_remove_all(), Ok(()));
            assert_eq!(None, c.property_get(c.uuid(), "blah"));

            let sub = c.property_get_subject(c.uuid());
            assert!(sub.is_some());
            let sub = sub.unwrap();
            assert_eq!(0, sub.len());

            let all = c.property_get_all();
            assert_eq!(0, all.len());
        }

        #[test]
        fn client_callbacks() {
            let timeout = std::time::Duration::from_millis(10);
            let prop1 = Property::new("foo", None);
            let prop2 = Property::new(
                "http://churchofrobotron.com/2084",
                Some("robot apocalypse".into()),
            );

            let (mut c1, _) = Client::new("client1", ClientOptions::NO_START_SERVER).unwrap();
            let (c2, _) = Client::new("client2", ClientOptions::NO_START_SERVER).unwrap();
            let (sender, receiver): (Sender<PropertyChangeOwned>, _) = channel();
            assert_eq!(
                Ok(()),
                c1.register_property_change_handler(ClosurePropertyChangeHandler::new(
                    move |change| {
                        assert_eq!(Ok(()), sender.send(change.into()));
                    }
                ))
            );

            //must activate to get callbacks
            let ac = c1.activate_async((), ()).unwrap();

            assert_eq!(c2.property_set(c2.uuid(), "blah", &prop1), Ok(()));
            let r = receiver.recv_timeout(timeout);
            assert_eq!(
                Ok(PropertyChangeOwned::Created {
                    subject: c2.uuid(),
                    key: "blah".into()
                }),
                r
            );

            //doesn't matter which client is used to set or remove the property
            assert_eq!(
                ac.as_client().property_set(c2.uuid(), "blah", &prop2),
                Ok(())
            );
            let r = receiver.recv_timeout(timeout);
            assert_eq!(
                Ok(PropertyChangeOwned::Changed {
                    subject: c2.uuid(),
                    key: "blah".into()
                }),
                r
            );

            assert_eq!(c2.property_remove(c2.uuid(), "blah"), Ok(()));
            let r = receiver.recv_timeout(timeout);
            assert_eq!(
                Ok(PropertyChangeOwned::Deleted {
                    subject: c2.uuid(),
                    key: "blah".into()
                }),
                r
            );

            assert_eq!(c2.property_set(c2.uuid(), "blah", &prop1), Ok(()));
            assert_eq!(
                c2.property_set(ac.as_client().uuid(), "mutant", &prop2),
                Ok(())
            );
            let r = receiver.recv_timeout(timeout);
            assert_eq!(
                Ok(PropertyChangeOwned::Created {
                    subject: c2.uuid(),
                    key: "blah".into()
                }),
                r
            );
            let r = receiver.recv_timeout(timeout);
            assert_eq!(
                Ok(PropertyChangeOwned::Created {
                    subject: ac.as_client().uuid(),
                    key: "mutant".into()
                }),
                r
            );
        }

        #[test]
        #[should_panic]
        fn double_register() {
            let (mut c, _) = Client::new("client1", ClientOptions::NO_START_SERVER).unwrap();
            assert_eq!(
                Ok(()),
                c.register_property_change_handler(ClosurePropertyChangeHandler::new(|_| {}))
            );
            let _panic =
                c.register_property_change_handler(ClosurePropertyChangeHandler::new(|_| {}));
        }
    }
}
