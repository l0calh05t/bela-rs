use std::ops::{Deref, DerefMut};

/// Internal wrapper for `bela_sys::BelaInitSettings` implementing
/// `Deref` and `DerefMut` with `Target = bela_sys::BelaInitSettings`.
#[cfg(feature = "static")]
pub(crate) struct InitSettings(bela_sys::BelaInitSettings);

/// Internal wrapper for `bela_sys::BelaInitSettings` implementing
/// `Deref` and `DerefMut` with `Target = bela_sys::BelaInitSettings`.
#[cfg(not(feature = "static"))]
pub(crate) struct InitSettings(*mut bela_sys::BelaInitSettings);

impl Default for InitSettings {
    fn default() -> InitSettings {
        // When linking against the static Bela library, InitSettings
        // contains a bela_sys::BelaInitSettings directly, initialized
        // using bela_sys::Bela_defaultSettings.
        #[cfg(feature = "static")]
        let settings = unsafe {
            let mut settings = std::mem::MaybeUninit::<bela_sys::BelaInitSettings>::uninit();
            bela_sys::Bela_defaultSettings(settings.as_mut_ptr());
            settings.assume_init()
        };

        // When linking against the dynamic Bela library, InitSettings
        // contains a pointer to bela_sys::BelaInitSettings, allocated
        // via bela_sys::Bela_InitSettings_alloc and initialized using
        // bela_sys::Bela_defaultSettings for ABI compatibility.
        #[cfg(not(feature = "static"))]
        let settings = unsafe {
            let settings = bela_sys::Bela_InitSettings_alloc();
            assert!(!settings.is_null());
            bela_sys::Bela_defaultSettings(settings);
            settings
        };

        InitSettings(settings)
    }
}

#[cfg(not(feature = "static"))]
impl Drop for InitSettings {
    fn drop(&mut self) {
        // When linking against the dynamic Bela library, the settings
        // object allocated in Default::default, must be freed using
        // bela_sys::Bela_InitSettings_free
        unsafe {
            bela_sys::Bela_InitSettings_free(self.0);
        }
    }
}

impl Deref for InitSettings {
    type Target = bela_sys::BelaInitSettings;
    #[cfg(feature = "static")]
    fn deref(&self) -> &Self::Target {
        &self.0
    }

    #[cfg(not(feature = "static"))]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl DerefMut for InitSettings {
    #[cfg(feature = "static")]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }

    #[cfg(not(feature = "static"))]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}
