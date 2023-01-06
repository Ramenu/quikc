use std::fs;

use crate::compiler;


pub fn retrieve_source_files(dir: &str, source_files: &mut Vec<String>) 
{
    let paths = fs::read_dir(dir).expect("Failed to read from directory");
    
    for path in paths {
        let retrieved_path = path.unwrap().path();
        let path_str = retrieved_path.to_str().unwrap().to_string();
        if compiler::is_cpp_source_file(&path_str) || compiler::is_c_source_file(&path_str) {
            source_files.push(path_str);
        }
    }
}