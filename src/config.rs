pub const VERSION: u8 = 0;

#[cfg(not(any(
    feature = "maxCap64",
    feature = "maxCap128",
    feature = "maxCap256",
    feature = "maxCap512",
    feature = "maxCap1024",
    feature = "maxCap2048",
    feature = "maxCap4096"
)))]
pub const CAPACITY: usize = 32;

#[cfg(all(
    feature = "maxCap64",
    not(any(
        feature = "maxCap128",
        feature = "maxCap256",
        feature = "maxCap512",
        feature = "maxCap1024",
        feature = "maxCap2048",
        feature = "maxCap4096"
    ))
))]
pub const CAPACITY: usize = 64;

#[cfg(all(
    feature = "maxCap128",
    not(any(
        feature = "maxCap64",
        feature = "maxCap256",
        feature = "maxCap512",
        feature = "maxCap1024",
        feature = "maxCap2048",
        feature = "maxCap4096"
    ))
))]
pub const CAPACITY: usize = 128;

#[cfg(all(
    feature = "maxCap256",
    not(any(
        feature = "maxCap64",
        feature = "maxCap128",
        feature = "maxCap512",
        feature = "maxCap1024",
        feature = "maxCap2048",
        feature = "maxCap4096"
    ))
))]
pub const CAPACITY: usize = 256;

#[cfg(all(
    feature = "maxCap512",
    not(any(
        feature = "maxCap64",
        feature = "maxCap128",
        feature = "maxCap256",
        feature = "maxCap1024",
        feature = "maxCap2048",
        feature = "maxCap4096"
    ))
))]
pub const CAPACITY: usize = 512;

#[cfg(all(
    feature = "maxCap1024",
    not(any(
        feature = "maxCap64",
        feature = "maxCap128",
        feature = "maxCap256",
        feature = "maxCap512",
        feature = "maxCap2048",
        feature = "maxCap4096"
    ))
))]
pub const CAPACITY: usize = 1024;

#[cfg(all(
    feature = "maxCap2048",
    not(any(
        feature = "maxCap64",
        feature = "maxCap128",
        feature = "maxCap256",
        feature = "maxCap512",
        feature = "maxCap1024",
        feature = "maxCap4096"
    ))
))]
pub const CAPACITY: usize = 2048;

#[cfg(all(
    feature = "maxCap4096",
    not(any(
        feature = "maxCap64",
        feature = "maxCap128",
        feature = "maxCap256",
        feature = "maxCap512",
        feature = "maxCap1024",
        feature = "maxCap2048",
    ))
))]
pub const CAPACITY: usize = 4096;
