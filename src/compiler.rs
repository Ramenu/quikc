use std::{path::PathBuf, sync::atomic::AtomicBool};
use color_print::{cprintln, cformat};
use rayon::prelude::*;
use std::path::Path;

use crate::{buildtable::{BUILD_TABLE_OBJECT_FILE_DIRECTORY}, build::{Build}};


#[inline]
pub fn to_output_file(path : &mut PathBuf, directory : &str, ext : &str) -> String
{
    path.set_extension(ext);
    return format!("{}/{}", directory, path.file_name().unwrap().to_str().unwrap());
}

#[inline]
pub fn is_cpp_source_file(file : &String) -> bool
{
    return file.ends_with(".cpp") || file.ends_with(".cxx") || file.ends_with(".cc");
}

#[inline]
pub fn is_c_source_file(file : &String) -> bool
{
    return file.ends_with(".c");
}

// dont support msvc at the moment
#[inline]
pub fn is_gcc_or_clang(compiler_name : &str) -> bool
{
    return match compiler_name {
        "gcc"|"g++"|"clang"|"clang++" => true,
        _ => false
    };
}

pub fn compile_to_object_files(source_files : &Vec<String>,
                               build_info : &Build) -> bool
{
    let compilation_successful = AtomicBool::new(true);
    source_files.into_par_iter().for_each(|file| {
        cprintln!("<green><bold>Compiling </bold>'{}'...</green>", file);

        let mut out_file_path = PathBuf::from(file);
        let out = to_output_file(&mut out_file_path, BUILD_TABLE_OBJECT_FILE_DIRECTORY, "o");

        
        let output = build_info.execute_compiler_with_build_info(file)
                                       .arg(file)
                                       .arg("-c")
                                       .arg("-o")
                                       .arg(&out)
                                       .output()
                                       .expect("Failed to execute process");
        
        if !output.status.success() {
            let s = String::from_utf8_lossy(&output.stderr);
            eprintln!("{}\n{}", s, cformat!("<red><bold>error:</bold></red> Failed to compile '{}'\nTerminating compilation.", file));

            // If there is a object file present from earlier compilations, remove it so that
            // the next time the program is run, it will know that an error occurred so it can
            // recompile it.
            if Path::new(&out).exists() {
                std::fs::remove_file(&out).expect("Failed to remove object file from build directory");
            }
            compilation_successful.store(false, std::sync::atomic::Ordering::Relaxed);
            return;
        }
    });

    return compilation_successful.load(std::sync::atomic::Ordering::Relaxed);
}