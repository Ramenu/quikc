use build::Build;
use color_print::cprintln;


mod compiler;
mod walker;
mod buildtable;
mod linker;
mod defaultbuild;
mod build;

const SOURCE_DIRECTORY : &str = "./src";

fn main() 
{
    let build_config = Build::new();
    let mut source_files = Vec::new();
    let mut build_table = buildtable::BuildTable::new();

    walker::retrieve_source_files(SOURCE_DIRECTORY, 
                                  &mut source_files, 
                                  &build_config.get_compiler_name(), 
                                  &mut build_table);

    let compilation_successful = compiler::compile_to_object_files(&mut source_files, &build_config);

    if compilation_successful {
        let link_successful = linker::link_files(&build_config);
        let build_type = match build_config.is_debug_build() {
            true => "debug",
            false => "release"
        };
        if link_successful {
            cprintln!("<green><bold>Successfully built target {} [{} build]</bold></green>", 
                      retrieve_file_name(build_config.get_package_name()),
                      build_type);
        }
    }

    build_table.write();
}


fn retrieve_file_name(s : &str) -> String
{
    for (i, c) in s.chars().rev().enumerate() {
        if c == '/' {
            return s[(s.len() - i)..].to_string();
        }
    }
    return s.to_string();
}
