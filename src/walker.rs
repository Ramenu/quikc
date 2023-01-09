use std::{fs};

use color_print::cformat;

use crate::{compiler, buildtable::BuildTable};


/// Retrieves the source files that need to be compiled
pub fn retrieve_source_files(dir: &str, 
                             source_files: &mut Vec<String>, 
                             compiler_name : &str, 
                             build_table : &mut BuildTable) 
{
    let old_table = build_table.get_table().clone();
    let mut has_source_file = false;
    let paths = fs::read_dir(dir).expect("Failed to read from directory");
    
    // only append the c/c++ files that need to be recompiled into the vector
    for path in paths {
        let mut retrieved_path = path.unwrap().path();
        let path_str = retrieved_path.to_str().unwrap().to_string();
        if compiler::is_cpp_source_file(&path_str) || compiler::is_c_source_file(&path_str) {
            has_source_file = true;
            if build_table.needs_to_be_recompiled(&mut retrieved_path, compiler_name, &old_table) {
                source_files.push(path_str);
            }
        }
    }

    // If no source files were found, print an error and terminate the program as there is nothing
    // to do
    if !has_source_file {
        eprintln!("{}", cformat!("<bold><red>error</red></bold>: no source files found in '{}'. Terminating program.", dir));
        std::process::exit(1);
    }
}