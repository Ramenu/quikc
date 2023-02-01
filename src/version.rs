use const_format::concatcp;

const _BEGIN_V : usize = line!() as usize;
pub const VERSIONS : [&str; _END_V] = [
    concatcp!("0", ".", MINOR_VERSIONS[0], ".", PATCH_VERSIONS[0])
];
const _END_V : usize = (line!() as usize) - _BEGIN_V - 3;

const _BEGIN_MV : usize = line!() as usize;
pub const MINOR_VERSIONS : [&str; _END_MV] = [
    "3",
    "2",
    "1"
];
const _END_MV : usize = (line!() as usize) - _BEGIN_MV - 3;

const _BEGIN_PV : usize = line!() as usize;
pub const PATCH_VERSIONS : [&str; _END_PV] = [
    "0"
];
const _END_PV : usize = (line!() as usize) - _BEGIN_PV - 3;

#[cfg(feature = "quikc-nightly")]
pub const NIGHTLY_VERSION : &str = concatcp!(VERSIONS[0], "-nightly");