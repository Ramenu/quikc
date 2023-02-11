use std::{fs};

use color_print::{cformat, cprintln};

use crate::QuikcFlags;
use crate::flags;

use crate::{buildtable::BUILD_TABLE_OBJECT_FILE_DIRECTORY, build::{Build, Linker}};

#[inline]
pub fn use_default_linker_configuration(linker : &Linker) -> bool
{
    #[cfg(feature = "quikc-nightly")]
    {
        if let Some(true) = linker.append_args {
            return linker.args.is_some()
        }
    }
    linker.args.is_none()
}

/// Links the object files given in '/buildtable/obj' and produces
/// an executable file if the linker returned no errors. If nothing
/// went wrong, this function will return true. Otherwise, it will
/// terminate the program. The bool return value is only used for
/// testing purposes.
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

    if flags()&QuikcFlags::HIDE_OUTPUT == QuikcFlags::NONE {
        cprintln!("<green><bold>Linking executable</bold> '{}'...</green>", build_config.package.name);
    }

    let cmd = build_config.execute_linker_with_build_info()
                                    .args(object_files.iter())
                                    .arg("-o")
                                    .arg(&build_config.package.name)
                                    .output()
                                    .expect("Failed to execute linker");
    
    if !cmd.status.success() {
        let err_output = String::from_utf8_lossy(&cmd.stderr);
        eprintln!("{}\n{}", 
                    cformat!("<bold><red>error</red>:</bold> Failed to link executable '{}'", build_config.package.name), 
                    err_output);
        return false;
    }

    true
}