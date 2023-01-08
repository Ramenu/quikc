use std::{path::{PathBuf, Path}, fs::{File, self}, time::UNIX_EPOCH, process::{Command}};

use crate::{compiler::{self, INCLUDE_PATH}};


pub const BUILD_TABLE_DIRECTORY : &str = "./buildinfo";
pub const BUILD_TABLE_OBJECT_FILE_DIRECTORY : &str = "./buildinfo/obj";
pub const BUILD_TABLE_FILE : &str = "./buildinfo/table.toml";

pub struct BuildTable
{
    table : toml::value::Table
}


impl BuildTable
{

    pub fn new() -> BuildTable
    {

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

    fn get_file_dependencies(&self,
                             compiler_name : &str, 
                             source_file_name : &String) -> Vec<String>
    {
        let cmd_output = &Command::new(compiler_name)
                                            .arg(INCLUDE_PATH)
                                            .arg("-MM")
                                            .arg(source_file_name)
                                            .output()
                                            .expect("Failed to retrieve dependencies")
                                            .stdout;

        let dependencies_str_tmp = String::from_utf8_lossy(&cmd_output);
        
        let dependencies = move || {
            let mut count = 0;
            for (i, c) in dependencies_str_tmp.chars().enumerate() {
                if c == ' ' {
                    count += 1;
                }
                if count == 2 {
                    return dependencies_str_tmp[i + 1..].split_whitespace()
                                                        .map(|s| s.to_string()).collect::<Vec<String>>();
                }
            }
            return Vec::new();
        };

        return dependencies();
    }

    pub fn needs_to_be_recompiled(&mut self,
                                  source_file_path : &mut PathBuf,
                                  compiler_name : &str) -> bool
    {
        let source_file_name = source_file_path.to_str().unwrap().to_string();

        // Before checking for dependencies, check if the source file has changed first to prevent
        // unnecessary work. If it has, then the source file has to be recompiled
        if !self.file_modified_since_last_build(source_file_path, 
                                                &source_file_name,
                                                false) {

            // source file hasnt changed, so check the dependencies to see if any of them changed,
            // if so then we need to recompile
            let dependencies = self.get_file_dependencies(compiler_name, &source_file_name);

            for dependency in dependencies {
                // sometimes the compiler shows '\' for line breaks, so we need to ignore those
                if dependency != "\\" {
                    let mut dependency_path = PathBuf::from(&dependency);
                    if self.file_modified_since_last_build(&mut dependency_path, 
                                                            &dependency, 
                                                            true) {
                        return true;
                    }
                }
            }
            return false;

        }
        return true;

    }

    fn file_modified_since_last_build(&mut self, 
                                       source_file_path : &mut PathBuf, 
                                       source_file_name : &String,
                                       is_header_file : bool) -> bool
    {

        // retrieve source file's metadata
        let source_metadata = source_file_path.metadata().expect("Failed to retrieve metadata from file");

        // retrieve the time that has been elapsed since it was last modified (in seconds)
        let time = (source_metadata.modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_secs() / 6000) as i64;

        // check if value exists in the table, if so, compare the times,
        // if they are the same, then the source file has not been modified,
        // so recompilation is not necessary. Otherwise, it is
        if self.table.contains_key(source_file_name) {

            let old_value = self.table.get(source_file_name).unwrap().as_integer().unwrap();
            if old_value != time {
                self.table.insert(source_file_name.to_string(), toml::Value::Integer(time));
                return true;
            }
            
            // only check for object file if the source isnt a header
            if !is_header_file {
                let object_file = compiler::to_output_file(source_file_path, BUILD_TABLE_OBJECT_FILE_DIRECTORY, "o");
                
                // If the object file does exist, compilation was most likely successful, if not
                // then re-compilation is necessary
                if Path::new(&object_file).exists() {
                    return false;
                }
                return true;
            }
            return false;
        }

        self.table.insert(source_file_name.to_string(), toml::Value::Integer(time));
        return true;
    }


    #[inline]
    pub fn write(&self)
    {
        fs::write(BUILD_TABLE_FILE, toml::to_string(&self.table).unwrap()).expect("Failed to write to file");
    }
}