use std::{path::{PathBuf, Path}, fs::{File, self}, time::UNIX_EPOCH, process::{Command, Output}, io};


use sha2::{Sha256, Digest};

use crate::{compiler};


pub const BUILD_TABLE_DIRECTORY : &str = "./buildinfo";
pub const BUILD_TABLE_PREPROCESSOR_DIRECTORY : &str = "./buildinfo/cpp";
pub const BUILD_TABLE_OBJECT_FILE_DIRECTORY : &str = "./buildinfo/obj";
pub const BUILD_TABLE_FILE : &str = "./buildinfo/table.toml";

pub struct BuildTable
{
    table : toml::value::Table
}

#[inline]
fn run_preprocessor_on_file(compiler_name : &str, source_file : &str, out_file : &str) -> Output
{
    return Command::new(compiler_name)
                   .arg("-E")
                   .arg(source_file)
                   .arg("-o")
                   .arg(out_file)
                   .output()
                   .expect("Failed to execute preprocessor on source file");
}

impl BuildTable
{

    pub fn new() -> BuildTable
    {
        // Create build table directory if it doesnt exist
        if !Path::new(BUILD_TABLE_DIRECTORY).is_dir() {
            std::fs::create_dir(&BUILD_TABLE_DIRECTORY).expect("Failed to create directory");
        }

        // Create build preprocessor directory
        if !Path::new(BUILD_TABLE_PREPROCESSOR_DIRECTORY).is_dir() {
            std::fs::create_dir(BUILD_TABLE_PREPROCESSOR_DIRECTORY).expect("Failed to create build preprocessor directory");
        }

        // Create build object file directory
        if !Path::new(BUILD_TABLE_OBJECT_FILE_DIRECTORY).is_dir() {
            std::fs::create_dir(BUILD_TABLE_OBJECT_FILE_DIRECTORY).expect("Failed to create build object file directory");
        }

        if !Path::new(BUILD_TABLE_FILE).is_file() {
            File::create(BUILD_TABLE_FILE).expect("Failed to create file");
        }

        let file_contents = fs::read_to_string(BUILD_TABLE_FILE).expect("Failed to read from build table file");

        let table : toml::value::Table = toml::from_str(&file_contents).expect("Failed to parse build table file");

        return BuildTable {
            table
        };
    }


    pub fn needs_to_be_recompiled(&mut self, 
                                       source_file_path : &mut PathBuf, 
                                       compiler_name : &str) -> bool
    {
        // Elapsed time since the source file was edited in the table
        let source_file_name = source_file_path.to_str().unwrap().to_string();

        // retrieve source file's metadata
        let source_metadata = source_file_path.metadata().expect("Failed to retrieve metadata from file");

        // retrieve the time that has been elapsed since it was last modified (in seconds)
        let time = (source_metadata.modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_secs() / 6000) as i64;

        // check if value exists in the table, if so, compare the times,
        // if they are the same, then the source file has not been modified,
        // so recompilation is not necessary. Otherwise, it is
        if self.table.contains_key(&source_file_name) {
            let old_value = self.table.get(&source_file_name).unwrap().as_integer().unwrap();
            if old_value != time {
                self.table.insert(source_file_name, toml::Value::Integer(time));
                return true;
            }

            let object_file = compiler::to_output_file(source_file_path, BUILD_TABLE_OBJECT_FILE_DIRECTORY, "o");
            
            // If the object file does exist, compilation was most likely successful, if not
            // then re-compilation is necessary
            if Path::new(&object_file).exists() {

                // Last thing: run the preprocessor on the file to check if any of the
                // headers changed. If any of them did, then recompilation is necessary.
                // This is avoided and only done for the last phase because it is not as
                // efficient as checking the modification time.
                let build_preprocessor_file = compiler::to_output_file(source_file_path, BUILD_TABLE_PREPROCESSOR_DIRECTORY, "i");

                // If the preprocessor build file does not exist, then recompilation is necessary
                if !Path::new(&build_preprocessor_file).exists() {
                    run_preprocessor_on_file(compiler_name, &source_file_name, &build_preprocessor_file);
                    return true;
                }

                // If the preprocessor build file does exist, then create a preprocessed temporary 
                // of this file and compare their hashes
                let source_preprocessor_file = compiler::to_output_file(source_file_path, BUILD_TABLE_PREPROCESSOR_DIRECTORY, "ii");
                run_preprocessor_on_file(compiler_name, &source_file_name, &source_preprocessor_file);

                /* insert error handling for checking if preprocessor output returned no errors */

                let mut cpp_file_1 = File::open(&build_preprocessor_file).expect("Failed to open build preprocessor file");
                let mut cpp_file_2 = File::open(&source_preprocessor_file).expect("Failed to open source preprocessor file");

                let mut sha_1 = Sha256::new();
                let mut sha_2 = Sha256::new();
                io::copy(&mut cpp_file_1, &mut sha_1).expect("Failed to copy from build preprocessor file");
                io::copy(&mut cpp_file_2, &mut sha_2).expect("Failed to copy from source preprocessor file");

                let hash_1 = sha_1.finalize();
                let hash_2 = sha_2.finalize();

                if hash_1 != hash_2 {
                    let original = &source_preprocessor_file[..source_preprocessor_file.len() - 1];
                    fs::rename(&source_preprocessor_file, original).expect("Failed to rename file");
                    return true;
                }
                else {
                    fs::remove_file(&source_preprocessor_file).expect("Failed to delete source preprocessor file");
                }
                return false;
            }
        }

        self.table.insert(source_file_name, toml::Value::Integer(time));
        return true;
    }


    #[inline]
    pub fn write(&self)
    {
        fs::write(BUILD_TABLE_FILE, toml::to_string(&self.table).unwrap()).expect("Failed to write to file");
    }
}