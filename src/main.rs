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

const SOURCE_DIRECTORY : &str = "./src";

fn main() 
{
    let build_config = Build::new();
    let mut old_table = toml::value::Table::new();
    let mut source_files = Vec::new();
    let mut build_table = buildtable::BuildTable::new(&mut old_table);

    walker::retrieve_source_files(SOURCE_DIRECTORY, 
                                  &mut source_files, 
                                  &build_config.get_compiler_name(), 
                                  &mut build_table,
                                  &mut old_table);
    if !source_files.is_empty() {
        let compilation_successful = compiler::compile_to_object_files(&mut source_files, &build_config);

        if compilation_successful {
            let link_successful = linker::link_files(&build_config);
            if link_successful {
                success(&build_config);
            }
        }
    }
    else {
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
            retrieve_file_name(build_config.get_package_name()),
            build_type);
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
