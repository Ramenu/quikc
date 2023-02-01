use std::{collections::HashMap, process::Command, path::{PathBuf, Path}, sync::atomic::AtomicBool};

use color_print::{cprintln, cformat};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::{walker, SOURCE_DIRECTORY, buildtable::{BuildTable, BUILD_TABLE_ASM_DIRECTORY}, build::Build, flags, QuikcFlags, compiler::{self, INCLUDE_PATH_FLAG}};


#[inline]
pub fn use_default_assembler_configuration(asm_args : &Option<Vec<String>>) -> bool
{
    asm_args.is_none() || asm_args.as_ref().unwrap().is_empty()
}

fn compile_to_asm_files(source_files : &Vec<&String>,
                        build : &Build) -> bool
{
    let show_assembling_progress = flags()&QuikcFlags::HIDE_OUTPUT == QuikcFlags::NONE;
    #[cfg(test)]
        let compilation_error = AtomicBool::new(false);
    source_files.into_par_iter().for_each(|file| {
        if show_assembling_progress {
            cprintln!("<green><bold>Assembling </bold>'{}'...</green>", file);
        }

        let out_file = compiler::to_output_file(&PathBuf::from(&file), BUILD_TABLE_ASM_DIRECTORY, "s");
        let output = build.execute_assembler_with_build_info(&file)
                                        .args([INCLUDE_PATH_FLAG, &file, "-S", "-o", &out_file])
                                        .output()
                                        .expect("Failed to execute assembler");

        if !output.status.success() {
            let s = String::from_utf8_lossy(&output.stderr);
            // print assembler error
            eprintln!("{}\n{}", s, cformat!("<bold><red>error</red>:</bold> Failed to assemble '{}'\nTerminating program.", file));

            // remove assembly file if it existed
            if Path::new(&out_file).exists() {
                std::fs::remove_file(&out_file).expect("Failed to remove output file");
            }

            #[cfg(not(test))]
                std::process::exit(1);
            // we don't want to exit if we are running tests
            #[cfg(test)]
                compilation_error.store(true, std::sync::atomic::Ordering::Relaxed);
            
        }
    });
    #[cfg(test)]
        return !compilation_error.load(std::sync::atomic::Ordering::Relaxed);
    #[cfg(not(test))]
        true
}

pub fn assemble_files(files : &Vec<&String>, 
                      build : &Build,
                      build_table : &mut BuildTable,
                      old_table : &HashMap<String, u64>) -> bool
{
    // If there are no files specified, then just assume the user wants to
    // assemble all of the source files
    if files.is_empty() {
        let source_files = walker::retrieve_source_files(SOURCE_DIRECTORY, build_table, old_table);
        if source_files.is_empty() {
            return true;
        }
        return compile_to_asm_files(&source_files.iter().collect(), build);
    }
    compile_to_asm_files(files, build)
}