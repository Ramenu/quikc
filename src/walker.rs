use std::{fs::{self}, path::Path, collections::HashMap};

use color_print::cformat;

use crate::{compiler::{self, to_output_file}, buildtable::{BuildTable, BUILD_TABLE_OBJECT_FILE_DIRECTORY}};

const SOURCE_EXTENSIONS : [&str;4] = ["c", "cpp", "cc", "cxx"];

/// Returns true if a source dependency that existed
/// from last compilation is not found in 'dir'. This
/// is for checking if a source file was deleted. If it
/// was, then the entire project must be recompiled.
#[inline]
fn source_dependency_missing(dir : &str, build_table : &mut BuildTable) -> bool
{
    let paths = fs::read_dir(BUILD_TABLE_OBJECT_FILE_DIRECTORY)
                            .expect("Failed to read from build table object file directory");

    let mut source_file_path = String::new();
    for path in paths {
        let path = path.unwrap().path();
        let mut exists = false;
        let object_file_path = &path.to_str().unwrap().to_string();

        for ext in SOURCE_EXTENSIONS {
            source_file_path = to_output_file(&path, dir, ext);
            if Path::new(&source_file_path).exists() {
                exists = true;
                break;
            }
        }

        if !exists {
            // Remove object file since it is no longer in the source directory 
            fs::remove_file(object_file_path).expect("Failed to remove object file");
            build_table.erase(&source_file_path);
            return true;
        }
    }

    false

}

/// Retrieves the source files that need to be compiled
pub fn retrieve_source_files(dir: &str, 
                             build_table : &mut BuildTable,
                             old_table : &HashMap<String, u64>) -> Vec<String>
{
    let mut source_files = Vec::new();
    let mut has_source_file = false;
    let paths = fs::read_dir(dir).expect("Failed to read from directory");
    let source_dependency_missing = source_dependency_missing(dir, build_table);
    let mut source_file_needs_to_be_recompiled = false;
    
    // only append the c/c++ files that need to be recompiled into the vector
    for path in paths.flatten() {
        let retrieved_path = path.path();
        let path_str = retrieved_path.to_str().unwrap();
        if compiler::is_cpp_source_file(path_str) || compiler::is_c_source_file(path_str) {
            has_source_file = true;

            if source_dependency_missing ||
               build_table.needs_to_be_recompiled(&retrieved_path, old_table) {
                source_file_needs_to_be_recompiled = true;
                source_files.push(path_str.to_string());
            }
        }
    }

    if source_file_needs_to_be_recompiled {
        build_table.set_any_dependencies_changed(true);
    }

    // If no source files were found, print an error and terminate the program as there is nothing
    // to do
    if !has_source_file {
        eprintln!("{}", cformat!("<bold><red>error</red></bold>: no source files found in '{}'. Terminating program.", dir));
        std::process::exit(1);
    }

    source_files
}

