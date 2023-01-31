use color_print::cprintln;



#[inline]
#[allow(dead_code)]
pub fn warning(msg : &str)
{
    cprintln!("<bold><yellow>warning</yellow>:</bold> {}", msg);
}

#[allow(dead_code)]
pub fn error(msg : &str)
{
    cprintln!("<bold><red>error</red>:</bold> {}", msg);
}