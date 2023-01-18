use std::{path::PathBuf, sync::atomic::AtomicBool, process::{Command}, io::ErrorKind};
use color_print::{cprintln, cformat};
use rayon::prelude::*;
use std::path::Path;

use crate::{buildtable::{BUILD_TABLE_OBJECT_FILE_DIRECTORY, BUILD_TABLE_DEPS_DIRECTORY}, build::{Build}};

pub const INCLUDE_PATH_FLAG : &str = "-I./include";
pub const INCLUDE_PATH : &str = "./include";

#[inline]
pub fn is_header_file(file : &str) -> bool
{
    if file.ends_with(".h") {
        return true;
    }
    return file.ends_with(".hpp") || file.ends_with(".hxx") || file.ends_with(".hh");
}
#[inline]
pub fn to_output_file(path : &mut PathBuf, directory : &str, ext : &str) -> String
{
    path.set_extension(ext);
    return format!("{}/{}", directory, path.file_name().unwrap().to_str().unwrap());
}

#[inline]
pub fn is_cpp_source_file(file : &str) -> bool
{
    return file.ends_with(".cpp") || file.ends_with(".cxx") || file.ends_with(".cc");
}

#[inline]
pub fn is_c_source_file(file : &str) -> bool
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

#[inline]
pub fn use_default_compiler_configuration(compiler_args : &Option<Vec<String>>) -> bool
{
    if compiler_args.is_some() {
        if !compiler_args.as_ref().unwrap().is_empty() {
            return false;
        }
    }
    return true;
}

/// Selects a default compiler, should be called only if a compiler has not
/// been specified in the 'build.toml' file. Available default compilers to
/// choose from include: gcc, clang, g++, clang++
pub fn select_default_compiler() -> &'static str
{
    if match Command::new("gcc").output() {
        Ok(_) => true,
        Err(e) => if let ErrorKind::NotFound = e.kind() { false } else { true }
    } { return "gcc" }

    if match Command::new("clang").output() {
        Ok(_) => true,
        Err(e) => if let ErrorKind::NotFound = e.kind() { false } else { true }
    } { return "clang" }

    if match Command::new("g++").output() {
        Ok(_) => true,
        Err(e) => if let ErrorKind::NotFound = e.kind() { false } else { true }
    } { return "g++" }

    if match Command::new("clang++").output() {
        Ok(_) => true,
        Err(e) => if let ErrorKind::NotFound = e.kind() { false } else { true }
    } { return "clang++" }

    eprintln!("{}", cformat!("<bold><red>error</red>:</bold> Could not find a default compiler to use.
                            Please specify your own in the 'build.toml' file"));
    std::process::exit(1);
}

pub fn compile_to_object_files(source_files : &Vec<String>,
                               build_info : &Build) -> bool
{
    let compilation_successful = AtomicBool::new(true);
    source_files.into_par_iter().for_each(|file| {
        cprintln!("<green><bold>Compiling </bold>'{}'...</green>", file);

        let mut out_file_path = PathBuf::from(file);
        let out = to_output_file(&mut out_file_path, BUILD_TABLE_OBJECT_FILE_DIRECTORY, "o");
        let dep_name = to_output_file(&mut out_file_path, BUILD_TABLE_DEPS_DIRECTORY, "d");

        Command::new(build_info.get_compiler_name())
                .args([INCLUDE_PATH_FLAG, file, "-MM", "-o", &dep_name])
                .spawn()
                .expect("Failed to spawn process");
        
        let output = build_info.execute_compiler_with_build_info(file)
                                       .args([INCLUDE_PATH_FLAG, file, "-c", "-o", &out])
                                       .output()
                                       .expect("Failed to execute process");
        
        if !output.status.success() {
            let s = String::from_utf8_lossy(&output.stderr);
            eprintln!("{}\n{}", s, cformat!("<bold><red>error</red>:</bold> Failed to compile '{}'\nTerminating compilation.", file));

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
    // TODO: If the compilation failed, terminate the program (only do this after updating the build table)
    return compilation_successful.load(std::sync::atomic::Ordering::Relaxed);
}