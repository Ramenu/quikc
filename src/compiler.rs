use std::{process::{Command, exit}, sync::atomic::AtomicUsize, path::PathBuf};
use color_print::{cprintln, cformat};
use rayon::prelude::*;
use std::sync::atomic::Ordering;
use std::path::Path;

pub struct Compiler<'a>
{
    name : &'static str,
    source_files : &'a Vec<String>,
    compile_flags : &'static str,
    out : &'static str
}

impl<'a> Compiler<'a>
{
    pub fn new(name : &'static str, 
               source_files : &'a Vec<String>, 
               compile_flags : &'static str,
               out : &'static str) -> Compiler<'a>
    {
        if !Path::new(out).is_dir() {
            std::fs::create_dir(out).expect("Failed to create directory");
        }

        return Compiler {
            name,
            source_files,
            compile_flags,
            out
        };
    }
}

#[inline]
pub fn to_object_file(path : &mut PathBuf, out : &str) -> String
{
    path.set_extension("o");
    return format!("{}/{}", out, path.file_name().unwrap().to_str().unwrap());
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

pub fn compile_to_object_files(compile_info : &Compiler) 
{
    compile_info.source_files.into_par_iter().for_each(|file| {
        cprintln!("<green><bold>Compiling </bold>'{}'...</green>", file);

        let mut out_file_path = PathBuf::from(file);
        let out = to_object_file(&mut out_file_path, compile_info.out);

        let output = Command::new(compile_info.name)
                                     .arg(&file)
                                     .arg("-fdiagnostics-color")
                                     .arg("-c")
                                     .arg("-o")
                                     .arg(out)
                                     .output()
                                     .expect("Failed to execute process");


        if !output.status.success() {
            let s = String::from_utf8_lossy(&output.stderr);
            eprintln!("{}\n{}", s, cformat!("<red><bold>ERROR:</bold></red> Failed to compile '{}'\nTerminating compilation.", file));
        }
        return;
    });
}