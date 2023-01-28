use std::{path::Path, collections::HashMap};

use build::Build;
use color_print::cprintln;
use bitflags::bitflags;


mod compiler;
mod walker;
mod buildtable;
mod linker;
mod defaultbuild;
mod build;
mod version;
#[cfg(test)]
    mod test;
#[cfg(test)]
    mod benchmark;

const SOURCE_DIRECTORY : &str = "./src";

bitflags! {
    pub struct QuikcFlags : u32 {
        const NONE = 0;
        const HIDE_VERBOSE_OUTPUT = 1;
    }
}

fn main() 
{
    let quikc_flags = parse_args();
    let build_config = Build::new(quikc_flags);
    let mut old_table = HashMap::new();
    let mut source_files = Vec::new();
    let mut build_table = buildtable::BuildTable::new(&mut old_table);

    walker::retrieve_source_files(SOURCE_DIRECTORY, 
                                  &mut source_files, 
                                  &mut build_table,
                                  &mut old_table);
    if !source_files.is_empty() {
        let compilation_successful = compiler::compile_to_object_files(&source_files, &build_config);

        if compilation_successful {
            link(&build_config);
        }
        return;
    }
    // Check if the binary exists, if not we need to relink
    if !Path::new(&build_config.get_package_name()).is_file() {
        link(&build_config);
        return;
    }
    success(&build_config);

}


fn parse_args() -> QuikcFlags
{
    let args = std::env::args().collect::<Vec<String>>();
    let mut flags = QuikcFlags::NONE;
    for arg in args {
        let mut starts_flag = false;
        for c in arg.chars() {
            if c == '-' && !starts_flag {
                starts_flag = true;
                continue;
            }
            else if !starts_flag {
                break;
            }
            if starts_flag {
                match c {
                    // show version and terminate program
                    'v' => {
                        #[cfg(feature = "quikc-nightly")]
                            println!("quikc-nightly v{}", version::NIGHTLY_VERSIONS[0]);
                        #[cfg(not(feature = "quikc-nightly"))]
                            println!("quikc v{}", version::VERSIONS[0]);
                        std::process::exit(0);
                    },
                    // hide verbose output
                    'h' => flags |= QuikcFlags::HIDE_VERBOSE_OUTPUT,
                    _ => ()
                };
            }
        }
    }
    flags
}


fn link(build_config : &Build)
{
    let link_successful = linker::link_files(build_config);
    if link_successful {
        success(build_config);
    }
}

fn success(build_config : &Build)
{
    let build_type = match build_config.is_debug_build() {
        true => "debug",
        false => "release"
    };

    cprintln!("<green><bold>Successfully built target {} [{} build]</bold></green>", 
            build_config.get_package_name(),
            build_type);
}
