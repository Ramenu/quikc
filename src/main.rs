use std::{path::Path, collections::HashMap};

use assembler::assemble_files;
use build::Build;
use buildtable::BuildTable;
use color_print::{cprintln, cformat};
use bitflags::bitflags;
use once_cell::sync::OnceCell;

use crate::logger::error;


mod compiler;
mod walker;
mod buildtable;
mod linker;
mod defaultbuild;
mod build;
mod version;
mod logger;
mod example;
mod assembler;

#[cfg(test)]
    mod test;
#[cfg(test)]
    mod benchmark;

const SOURCE_DIRECTORY : &str = "./src";

bitflags! {
    pub struct QuikcFlags : u32 {
        const NONE = 0;
        const HIDE_VERBOSE_OUTPUT = 1 << 0;
        const DO_NOT_LINK = 1 << 1;
        const HIDE_OUTPUT = 1 << 2;
        const SHOW_VERSION = 1 << 3;
        const ASSEMBLE = 1 << 4;
    }
}

static INSTANCE : OnceCell<QuikcFlags> = OnceCell::new();

#[inline]
pub fn flags() -> QuikcFlags {
    *INSTANCE.get().unwrap()
}

#[cfg(test)]
pub fn set_flags() {
    INSTANCE.set(QuikcFlags::NONE).unwrap();
}

fn main() 
{
    INSTANCE.set(parse_args()).unwrap();
    let build_config = Build::new();
    let mut old_table = HashMap::new();
    let mut build_table = buildtable::BuildTable::new(&mut old_table, true);

    let source_files = walker::retrieve_source_files(SOURCE_DIRECTORY, 
                                                                    &mut build_table,
                                                                    &old_table);
    if !source_files.is_empty() {
        let compilation_successful = compiler::compile_to_object_files(&source_files, &build_config);

        if compilation_successful {
            if flags()&QuikcFlags::DO_NOT_LINK == QuikcFlags::NONE {
                link(&build_config);
            } else {
                success(&build_config);
            }
        }
        return;
    }
    // Check if the binary exists, if not we need to relink
    if !Path::new(&build_config.package.name).is_file() && flags()&QuikcFlags::DO_NOT_LINK == QuikcFlags::NONE {
        link(&build_config);
        return;
    }
    success(&build_config);

}

#[inline]
fn parse_args() -> QuikcFlags
{
    let args = std::env::args().collect::<Vec<String>>();
    let mut files_to_assemble = Vec::new();
    let mut flags = QuikcFlags::NONE;

    for arg in &args {
        let mut starts_flag = false;
        for c in arg.chars() {
            if c == '-' && !starts_flag {
                starts_flag = true;
                continue;
            }
            else if !starts_flag && flags&QuikcFlags::ASSEMBLE == QuikcFlags::NONE {
                break;
            }
            if starts_flag {
                match c {
                    // show version 
                    'v' => {
                        // we don't want to show it more than once
                        if flags&QuikcFlags::SHOW_VERSION == QuikcFlags::NONE {
                            flags |= QuikcFlags::SHOW_VERSION;
                            #[cfg(feature = "quikc-nightly")]
                                println!("quikc-nightly v{}", version::NIGHTLY_VERSION);
                            #[cfg(not(feature = "quikc-nightly"))]
                                println!("quikc v{}", version::VERSIONS[0]);
                        }
                    },
                    // hide verbose output
                    'h' => {
                        // if '-hh' is specified, then do not show any output at all, with the exception of errors
                        // and compiler/linker output
                        if flags&QuikcFlags::HIDE_VERBOSE_OUTPUT == QuikcFlags::HIDE_VERBOSE_OUTPUT {
                            flags |= QuikcFlags::HIDE_OUTPUT;
                        }
                        else {
                            flags |= QuikcFlags::HIDE_VERBOSE_OUTPUT
                        }
                    },
                    // do not link after compiling
                    'c' => flags |= QuikcFlags::DO_NOT_LINK,
                    'S' => {
                        flags |= QuikcFlags::ASSEMBLE;
                        continue;
                    },
                    _ => {
                        error("unknown option specified");
                        std::process::exit(1);
                    }
                };
            }
            // If this is not a flag, and the assembler flag is set, then we can assume
            // this is a file that the user wants an assembly output of
            else if flags&QuikcFlags::ASSEMBLE == QuikcFlags::ASSEMBLE {
                if !Path::new(arg).is_file() {
                    eprintln!("{}", cformat!("<bold><red>error</red>:</bold> failed to disassemble '{}'. File does not exist", arg));
                    std::process::exit(1);
                }
                files_to_assemble.push(arg);
            }
        }
    }

    // user just wanted to check the version, exit here.
    if flags == QuikcFlags::SHOW_VERSION {
        std::process::exit(0);
    }

    if flags&QuikcFlags::ASSEMBLE == QuikcFlags::ASSEMBLE {
        INSTANCE.set(flags).unwrap();
        let build = Build::new();
        let mut old_table = HashMap::new();
        let mut build_table = BuildTable::new(&mut old_table, false);
        assemble_files(&files_to_assemble, &build, &mut build_table, &old_table);

        let build_type = match build.package.debug_build {
            true => "debug",
            false => "release"
        };

        if flags&QuikcFlags::HIDE_OUTPUT == QuikcFlags::NONE {
            if files_to_assemble.len() == 1 {
                cprintln!("<green><bold>Successfully assembled source file: '{}' [{}]</bold></green>", files_to_assemble[0], build_type);
            }
            else {
                cprintln!("<green><bold>Successfully assembled source files [{}]</bold></green>", build_type);
            }
        }
        std::process::exit(0);
    }

    flags
}

#[inline]
fn link(build_config : &Build)
{
    let link_successful = linker::link_files(build_config);
    if link_successful {
        success(build_config);
    }
}

#[inline]
fn success(build_config : &Build)
{
    if flags()&QuikcFlags::HIDE_OUTPUT == QuikcFlags::HIDE_OUTPUT {
        return;
    }

    let build_type = match build_config.package.debug_build {
        true => "debug",
        false => "release"
    };

    if flags()&QuikcFlags::DO_NOT_LINK == QuikcFlags::DO_NOT_LINK {
        cprintln!("<green><bold>Successfully compiled source files to object files [{}]</bold></green>", 
                    build_type);
        return;
    }

    cprintln!("<green><bold>Successfully built target {} [{} build]</bold></green>", 
                    build_config.package.name,
                    build_type);
}
