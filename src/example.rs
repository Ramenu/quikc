use std::{fs::{File}, io::{self, BufReader, BufRead}};

use color_print::cprintln;

use crate::build::BUILD_CONFIG_FILE;


pub enum FieldType
{
    CompilerArgs,
    LinkerArgs
}

fn read_lines(file_name : &str) -> io::Result<io::Lines<BufReader<File>>>
{
    let file = File::open(file_name)?;
    Ok(io::BufReader::new(file).lines())
}

fn get_field_parent(field : FieldType) -> &'static str
{
    match field {
        FieldType::CompilerArgs => "[compiler]",
        FieldType::LinkerArgs => "[linker]"
    }
}

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