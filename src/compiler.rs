use std::{path::PathBuf, process::{Command}, io::ErrorKind};

#[cfg(test)]
    use std::sync::atomic::AtomicBool;
#[cfg(test)]
    use std::sync::atomic::Ordering;
use color_print::{cprintln, cformat};
use rayon::prelude::*;
use std::path::Path;
#[cfg(feature = "quikc-nightly")] 
    use std::process::Stdio;
use crate::flags;
use crate::QuikcFlags;


use crate::{buildtable::{BUILD_TABLE_OBJECT_FILE_DIRECTORY, BUILD_TABLE_DEPS_DIRECTORY}, build::{Build, Compiler}};

pub const INCLUDE_PATH_FLAG : &str = "-I./include";
pub const INCLUDE_PATH : &str = "./include";

#[inline]
pub fn is_header_file(file : &str) -> bool
{
    if file.ends_with(".h") {
        return true;
    }
    file.ends_with(".hpp") || file.ends_with(".hxx") || file.ends_with(".hh")
}
#[inline]
pub fn to_output_file(path : &PathBuf, directory : &str, ext : &str) -> String
{
    // path.file_prefix() is currently unstable so cant use that. file_stem should
    // work for our use case though.
    format!("{}/{}.{}", directory, path.file_stem().unwrap().to_str().unwrap(), ext)
}

#[inline]
pub fn is_cpp_source_file(file : &str) -> bool
{
    file.ends_with(".cpp") || file.ends_with(".cxx") || file.ends_with(".cc")
}

#[inline]
pub fn is_c_source_file(file : &str) -> bool
{
    file.ends_with(".c")
}

// dont support msvc at the moment
#[inline]
pub fn is_gcc_or_clang(compiler_name : &str) -> bool
{
    matches!(compiler_name, "gcc"|"g++"|"clang"|"clang++")
}


#[inline]
pub fn use_default_compiler_configuration(compiler : &Compiler) -> bool
{
    #[cfg(feature = "quikc-nightly")] 
    {
        if let Some(true) = compiler.append_args {
            return compiler.args.is_some();
        }
    }
    compiler.args.is_none()
}

/// Selects a default compiler, should be called only if a compiler has not
/// been specified in the 'build.toml' file. Available default compilers to
/// choose from include: gcc, clang, g++, clang++
pub fn select_default_compiler() -> &'static str
{
    if match Command::new("gcc").output() {
        Ok(_) => true,
        Err(e) => !matches!(e.kind(), ErrorKind::NotFound)
    } { return "gcc" }

    if match Command::new("clang").output() {
        Ok(_) => true,
        Err(e) => !matches!(e.kind(), ErrorKind::NotFound)
    } { return "clang" }

    if match Command::new("g++").output() {
        Ok(_) => true,
        Err(e) => !matches!(e.kind(), ErrorKind::NotFound)
    } { return "g++" }

    if match Command::new("clang++").output() {
        Ok(_) => true,
        Err(e) => !matches!(e.kind(), ErrorKind::NotFound)
    } { return "clang++" }

    eprintln!("{}", cformat!("<bold><red>error</red>:</bold> Could not find a default compiler to use.
                            Please specify your own in the 'Build.toml' file"));
    std::process::exit(1);
}

pub fn compile_to_object_files(source_files : &Vec<String>,
                               build_info : &Build) -> bool
{
    let show_compiling_progress = flags()&QuikcFlags::HIDE_OUTPUT == QuikcFlags::NONE;
    #[cfg(test)]
        let compilation_error = AtomicBool::new(false);
        
    source_files.into_par_iter().for_each(|file| {
        if show_compiling_progress {
            cprintln!("<green><bold>Compiling </bold>'{}'...</green>", file);
        }

        let out_file_path = PathBuf::from(file);
        let out = to_output_file(&out_file_path, BUILD_TABLE_OBJECT_FILE_DIRECTORY, "o");
        let dep_name = to_output_file(&out_file_path, BUILD_TABLE_DEPS_DIRECTORY, "d");

        // Generate the file's dependencies
        Command::new(&build_info.compiler.compiler)
                .args([INCLUDE_PATH_FLAG, file, "-MM", "-o", &dep_name])
                .spawn()
                .expect("Failed to generate dependencies");

        // 'Include what you use' is currently a experimental feature, and not toggled by default 
        // since it can probably cause the program to not compile
        #[cfg(feature = "quikc-nightly")]
        if build_info.misc.toggle_iwyu.unwrap_or(false) {
            let standard = build_info.get_standard(file);
            let iwyu_cmd = Command::new("include-what-you-use")
                                          .args([standard, INCLUDE_PATH_FLAG, file])
                                          .stdout(Stdio::piped())
                                          .spawn()
                                          .expect("Failed to spawn 'include-what-you-use'");

            Command::new("iwyu-fix-includes")
                    .stdin(iwyu_cmd.stdout.unwrap())
                    .stdout(Stdio::null())
                    .spawn()
                    .expect("Failed to spawn 'iwyu-fix-includes'");
        }

        // Compile the file with the appropriate flags specified in the build
        let output = build_info.execute_compiler_with_build_info(file)
                                       .args([INCLUDE_PATH_FLAG, file, "-c", "-o", &out])
                                       .output()
                                       .expect("Failed to execute compiler");
        
        if !output.status.success() {
            let s = String::from_utf8_lossy(&output.stderr);
            eprintln!("{}\n{}", s, cformat!("<bold><red>error</red>:</bold> Failed to compile '{}'\nTerminating program.", file));

            // If there is a object file present from earlier compilations, remove it so that
            // the next time the program is run, it will know that an error occurred so it can
            // recompile it.
            if Path::new(&out).exists() {
                std::fs::remove_file(&out).expect("Failed to remove object file from build directory");
            }
            #[cfg(not(test))]
                std::process::exit(1);
            // We don't want to exit if we are running tests
            #[cfg(test)] 
            {
                compilation_error.store(true, Ordering::Relaxed);
            }
        }
    });
    #[cfg(test)]
        return !compilation_error.load(Ordering::Relaxed);
    #[cfg(not(test))]
        true
}