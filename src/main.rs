
mod compiler;
mod walker;
mod buildtable;

fn main() 
{
    let mut source_files = Vec::new();
    let compiler_name = "g++";
    let dir = "./testdir"; // directory to search
    let mut build_table = buildtable::BuildTable::new();

    walker::retrieve_source_files(dir, &mut source_files, compiler_name, &mut build_table);

    let compiler_info = compiler::Compiler::new(compiler_name, 
                                                          &source_files, 
                                                          "-std=c++17",
                                                          "main");
    compiler::compile_to_object_files(&compiler_info);
    build_table.write();
}
