#[cfg(feature = "quikc-nightly")]
use std::{fs::{File}, io::{self, BufReader, BufRead}};

#[cfg(feature = "quikc-nightly")]
use color_print::cprintln;

#[cfg(feature = "quikc-nightly")]
use crate::build::BUILD_CONFIG_FILE;


#[cfg(feature = "quikc-nightly")]
pub enum FieldType
{
    CompilerArgs,
    LinkerArgs
}

/// Returns the lines from the provided file.
#[cfg(feature = "quikc-nightly")]
fn read_lines(file_name : &str) -> io::Result<io::Lines<BufReader<File>>>
{
    let file = File::open(file_name)?;
    Ok(io::BufReader::new(file).lines())
}

/// Returns the parent (or object?) that the field belongs
/// to. 
#[cfg(feature = "quikc-nightly")]
fn get_field_parent(field : FieldType) -> &'static str
{
    match field {
        FieldType::CompilerArgs => "[compiler]",
        FieldType::LinkerArgs => "[linker]"
    }
}

/// Prints the missing field in the build config. Useful for showing
/// which fields to add to the build configuration in case of an error.
#[cfg(feature = "quikc-nightly")]
pub fn print_missing_field(field_missing : &str, field : FieldType)
{
    let mut line_num = 0;

    if let Ok(lines) = read_lines(BUILD_CONFIG_FILE) {
        let field_parent = get_field_parent(field);
        let mut found = false;
        for line in lines.flatten() {
            line_num += 1;
            if found {
                println!("             {}  {}", line_num + 1, line);
                break;
            }
            if line.trim() == field_parent {
                cprintln!("             {}  {}\n<g>Add this ->  {}      <u>{}</u></g>", line_num, line, line_num + 1, field_missing);
                found = true;
            }
        }
    }

}