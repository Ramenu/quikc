use std::{fs};

use crate::{compiler, buildtable::BuildTable};


/// Retrieves the source files that need to be compiled
pub fn retrieve_source_files(dir: &str, 
                             source_files: &mut Vec<String>, 
                             compiler_name : &str, 
                             build_table : &mut BuildTable) 
{
    let paths = fs::read_dir(dir).expect("Failed to read from directory");
    
    for path in paths {
        let mut retrieved_path = path.unwrap().path();
        let path_str = retrieved_path.to_str().unwrap().to_string();
        if compiler::is_cpp_source_file(&path_str) || compiler::is_c_source_file(&path_str) {
            if build_table.needs_to_be_recompiled(&mut retrieved_path, compiler_name) {
                source_files.push(path_str);
            }
        }
    }
}