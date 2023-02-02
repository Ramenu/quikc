use color_print::{cprintln, cformat};

/// Prints a warning message to stdout.
#[inline]
#[allow(dead_code)]
pub fn warning(msg : &str)
{
    cprintln!("<bold><yellow>warning</yellow>:</bold> {}", msg);
}

/// Prints an error message to stderr.
#[inline]
#[allow(dead_code)]
pub fn error(msg : &str)
{
    eprintln!("{}", cformat!("<bold><red>error</red>:</bold> {}", msg));
}