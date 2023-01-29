use color_print::cprintln;



#[inline]
pub fn warning(msg : &str)
{
    cprintln!("<bold><yellow>warning</yellow>:</bold> {}", msg);
}

pub fn error(msg : &str)
{
    cprintln!("<bold><red>error</red>:</bold> {}", msg);
}