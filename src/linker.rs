use std::{fs, path::{PathBuf}};

use color_print::{cformat, cprintln};

use crate::{buildtable::BUILD_TABLE_OBJECT_FILE_DIRECTORY, build::Build};

#[inline]
pub fn use_default_linker_configuration(linker_args : &Option<Vec<String>>) -> bool
{
    if linker_args.is_some() {
        if linker_args.as_ref().unwrap().len() > 0 {
            return false;
        }
    }
    return true;
}

pub fn link_files(build_config : &Build) -> bool
{
    let mut object_files = Vec::new();
    let dir = fs::read_dir(BUILD_TABLE_OBJECT_FILE_DIRECTORY)
                          .expect("Failed to read from object file directory");

    let binary_path = PathBuf::from(build_config.get_package_name());

    let mut need_to_relink = true;

    // If the binary does not exist, then no need to check
    if binary_path.exists() {
        need_to_relink = false;
        let binary_creation_time = binary_path.metadata()
                                                        .unwrap()
                                                        .created()
                                                        .unwrap();
        
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
    }
    else {
        for object_file in dir {
            let object_file_path = object_file.unwrap().path();
            let object_file_path_str = object_file_path.to_str().unwrap().to_string();
            object_files.push(object_file_path_str);
        }
    }

    if need_to_relink {
        let binary_path_str = binary_path.to_str().unwrap();
        cprintln!("<green><bold>Linking executable</bold> '{}'...</green>", binary_path_str);
        let cmd = build_config.execute_linker_with_build_info()
                                          .args(object_files.iter())
                                          .arg("-o")
                                          .arg(binary_path_str)
                                          .output()
                                          .expect("Failed to execute linker");
        
        if !cmd.status.success() {
            let err_output = String::from_utf8_lossy(&cmd.stderr);
            eprintln!("{}\n{}", 
                      cformat!("<bold><red>error</red>:</bold> Failed to link executable '{}'", binary_path_str), 
                      err_output);
            return false;
        }
    }
    return true;
}