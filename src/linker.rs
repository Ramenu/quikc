use std::{fs};

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

    for object_file in dir {
        let object_file_path = object_file.unwrap().path();
        let object_file_path_str = object_file_path.to_str().unwrap().to_string();
        object_files.push(object_file_path_str);
    }

    cprintln!("<green><bold>Linking executable</bold> '{}'...</green>", build_config.get_package_name());
    let cmd = build_config.execute_linker_with_build_info()
                                    .args(object_files.iter())
                                    .arg("-o")
                                    .arg(build_config.get_package_name())
                                    .output()
                                    .expect("Failed to execute linker");
    
    if !cmd.status.success() {
        let err_output = String::from_utf8_lossy(&cmd.stderr);
        eprintln!("{}\n{}", 
                    cformat!("<bold><red>error</red>:</bold> Failed to link executable '{}'", build_config.get_package_name()), 
                    err_output);
        return false;
    }

    return true;
}