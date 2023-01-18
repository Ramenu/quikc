use std::{path::Path, collections::HashMap};

use build::Build;
use color_print::cprintln;


mod compiler;
mod walker;
mod buildtable;
mod linker;
mod defaultbuild;
mod build;
#[cfg(test)]
    mod test;
#[cfg(test)]
    mod benchmark;

const SOURCE_DIRECTORY : &str = "./src";

fn main() 
{
    let build_config = Build::new();
    let mut old_table = HashMap::new();
    let mut source_files = Vec::new();
    let mut build_table = buildtable::BuildTable::new(&mut old_table);

    walker::retrieve_source_files(SOURCE_DIRECTORY, 
                                  &mut source_files, 
                                  &mut build_table,
                                  &mut old_table);
    if !source_files.is_empty() {
        let compilation_successful = compiler::compile_to_object_files(&mut source_files, &build_config);

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

fn link(build_config : &Build)
{
    let link_successful = linker::link_files(&build_config);
    if link_successful {
        success(&build_config);
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
