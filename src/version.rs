use const_format::concatcp;

pub const VERSIONS : [&str; 1] = [
    concatcp!("0", ".", MINOR_VERSIONS[0], ".", PATCH_VERSIONS[0])
];

pub const MINOR_VERSIONS : [&str; 2] = [
    "2",
    "1"
];

pub const PATCH_VERSIONS : [&str; 1] = [
    "0"
];

#[cfg(feature = "quikc-nightly")]
pub const NIGHTLY_VERSIONS : [&str; 1] = [
    concatcp!(VERSIONS[0], "-nightly")
];