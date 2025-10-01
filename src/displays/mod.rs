#![cfg(feature = "waveshare_18_amoled")]

pub mod waveshare_18_amoled;
pub use waveshare_18_amoled::*;

#[cfg(feature = "waveshare_18_amoled_async")]
pub mod waveshare_18_amoled_async;

#[cfg(feature = "waveshare_18_amoled_async")]
pub use waveshare_18_amoled_async::*;
