#[cfg(feature = "dynamic_loading")]
pub mod dynamic_loading;

#[cfg(not(feature = "dynamic_loading"))]
pub mod dynamic_linking;
