use std::path::PathBuf;

use color_print::cprintln;


mod compiler;
mod walker;
mod buildtable;
mod linker;

fn main() 
{
    let mut source_files = Vec::new();
    let compiler_name = "g++";
    let binary_name = "main";
    let dir = "./testdir"; // directory to search
    let mut build_table = buildtable::BuildTable::new();

    walker::retrieve_source_files(dir, &mut source_files, compiler_name, &mut build_table);

    let compiler_info = compiler::Compiler::new(compiler_name, 
                                                          &source_files, 
                                                          "-std=c++17");
    let compilation_successful = compiler::compile_to_object_files(&compiler_info);

    if compilation_successful {
        let link_successful = linker::link_files(compiler_name, "", binary_name);
        if link_successful {
            cprintln!("<green><bold>Successfully built target {}</bold></green>", retrieve_file_name(binary_name));
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
