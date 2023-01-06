use std::{process::Command, fs, path::{Path, PathBuf}};

use color_print::{cformat, cprintln};

use crate::buildtable::BUILD_TABLE_OBJECT_FILE_DIRECTORY;



pub fn link_files(compiler_name : &str,
                  linker_args : &str,
                  binary_name : &str) -> bool
{
    let mut object_files = Vec::new();
    let dir = fs::read_dir(BUILD_TABLE_OBJECT_FILE_DIRECTORY)
                          .expect("Failed to read from object file directory");
    let binary_path = PathBuf::from(binary_name);
    let binary_creation_time = binary_path.metadata()
                                                      .unwrap()
                                                      .created()
                                                      .unwrap();
    
    let mut need_to_relink = false;
    for object_file in dir {
        let object_file_path = object_file.unwrap().path();
        let object_file_path_str = object_file_path.to_str().unwrap().to_string();

        // If the binary exists, we have to check if any of the object files are newer than
        // the binary. If so, then that means we have to re-link it
        if binary_path.exists() && !need_to_relink {
            let object_file_creation_time = object_file_path.metadata()
                                                                        .unwrap()
                                                                        .created()
                                                                        .unwrap();
            if object_file_creation_time > binary_creation_time {
                need_to_relink = true;
            }
                                        
        }
        object_files.push(object_file_path_str);
    }

    if need_to_relink {
        cprintln!("<green><bold>Linking executable</bold> '{}'...</green>", binary_name);
        let command = Command::new(compiler_name)
                                            .args(object_files.iter())
                                            //.arg(linker_args)
                                            .arg("-o")
                                            .arg(binary_name)
                                            .output()
                                            .expect("Failed to execute linker");
        
        if !command.status.success() {
            let err_output = String::from_utf8_lossy(&command.stderr);
            eprintln!("{}\n{}", cformat!("<red><bold>error:</bold></red> Failed to link executable '{}'", binary_name), err_output);
            return false;
        }
    }
    return true;
}