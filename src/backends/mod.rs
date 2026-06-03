#[cfg(feature = "rar")]
pub mod rar;
pub mod sevenz;
pub mod streamc;
pub mod tar;
pub mod zip;

/// One entry reported by a `list` operation.
pub struct ListEntry {
    pub name: String,
    pub size: u64,
}

/// Clamp a user-supplied level into an inclusive range, falling back to a
/// default when none was given.
pub fn clamp_level(level: Option<i32>, min: i32, max: i32, default: i32) -> i32 {
    level.unwrap_or(default).clamp(min, max)
}
