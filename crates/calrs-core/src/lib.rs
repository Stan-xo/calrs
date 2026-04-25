//! calrs-core: core library for the calrs calendar application.

pub mod models;
pub mod profile;
pub mod storage;

pub const APP_NAME: &str = "calrs";
pub const DEFAULT_TIMEZONE: &str = "UTC"; // TODO : prendre fuseau horraire de l'appareil (PC, ou tel ...)
