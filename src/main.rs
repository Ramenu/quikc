
mod compiler;
mod walker;
mod build;
mod buildtable;

fn main() 
{
    let mut source_files = Vec::new();
    let dir = "./testdir"; // directory to search
    let out = "./buildinfo";
    let mut build_table = buildtable::BuildTable::new(out);

    walker::retrieve_source_files(dir, &mut source_files, out, &mut build_table);

    let compiler_info = compiler::Compiler::new("g++", 
                                                          &source_files, 
                                                          "-std=c++17",
                                                          out);
    compiler::compile_to_object_files(&compiler_info);
    build_table.write();
}
