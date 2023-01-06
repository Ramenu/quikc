
use std::process::Command;

use rayon::prelude::*;
mod compiler;
mod walker;

fn main() 
{
    let mut source_files = Vec::new();
    let dir = "./testdir"; // directory to search

    walker::retrieve_source_files(dir, &mut source_files);

    let compiler_info = compiler::Compiler::new("g++", 
                                                          &source_files, 
                                                          "-std=c++17",
                                                          "./buildinfo");
    compiler::compile_to_object_files(&compiler_info);
}
