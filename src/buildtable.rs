use std::{path::{PathBuf, Path}, fs::{File, self, Metadata}, time::UNIX_EPOCH, process::{Command}, sync::{atomic::{AtomicBool, Ordering}, Mutex, Arc}};

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::{compiler::{self, INCLUDE_PATH}};


pub const BUILD_TABLE_DIRECTORY : &str = "./buildinfo";
pub const BUILD_TABLE_OBJECT_FILE_DIRECTORY : &str = "./buildinfo/obj";
pub const BUILD_TABLE_FILE : &str = "./buildinfo/table.toml";

pub struct BuildTable
{
    table : toml::value::Table
}

#[inline]
fn get_duration_since_modified(metadata : &Metadata) -> i64
{
    return (metadata.modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_secs() / 2) as i64;
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
                                                        .map(|s| s.to_string())
                                                        .collect::<Vec<String>>();
                }
            }
            return Vec::new();
        };

        return dependencies();
    }

    pub fn needs_to_be_recompiled(&mut self,
                                  source_file_path : &mut PathBuf,
                                  compiler_name : &str,
                                  old_table : &toml::value::Table) -> bool
    {
        let recompile = AtomicBool::new(false);
        let source_file_name = source_file_path.to_str().unwrap().to_string();


        // check if source file has changed (note we still need to check if any dependencies have changed to update
        // the build table)
        let source_modified_duration = get_duration_since_modified(&source_file_path.metadata().unwrap());
        let time = toml::Value::Integer(source_modified_duration);
        if self.file_modified_since_last_build(source_file_path, 
                                               &source_file_name, 
                                               false, 
                                               source_modified_duration,
                                               old_table) {
            self.table.insert(source_file_name, time);
            return true;
        }
        else {
            let dependencies = self.get_file_dependencies(compiler_name, &source_file_name);
            let table = Arc::new(Mutex::new(self.table.clone()));

            // check if any dependencies have changed for the source file, if 1 has changed, we can
            // update all of their times
            dependencies.par_iter().for_each(|dependency| {
                // sometimes the compiler shows '\' for line breaks, so we need to ignore those
                if dependency != "\\" {
                    let mut dependency_path = PathBuf::from(dependency);
                    let source_metadata = dependency_path.metadata().expect("Failed to retrieve metadata from file");
                    let duration = get_duration_since_modified(&source_metadata);
                    if self.file_modified_since_last_build(&mut dependency_path, 
                                                            dependency, 
                                                            true,
                                                            duration,
                                                                old_table) {
                        recompile.store(true, Ordering::Relaxed);
                        let mut table = table.lock().unwrap();
                        table.insert(dependency.clone(), toml::Value::Integer(duration));
                    }
                }
            });
            self.table = table.lock().unwrap().clone(); //keep it commented just so if somethings not working uncomment
            return recompile.load(Ordering::Relaxed);
        }

    }

    fn file_modified_since_last_build(&self, 
                                       source_file_path : &mut PathBuf, 
                                       source_file_name : &String,
                                       is_header_file : bool,
                                       time : i64,
                                       old_table : &toml::value::Table) -> bool
    {

        // check if value exists in the table, if so, compare the times,
        // if they are the same, then the source file has not been modified,
        // so recompilation is not necessary. Otherwise, it is
        if old_table.contains_key(source_file_name) {
            let old_value = old_table.get(source_file_name).unwrap().as_integer().unwrap();

            if old_value != time {
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

        return true;
    }


    #[inline]
    pub fn write(&self)
    {
        fs::write(BUILD_TABLE_FILE, toml::to_string(&self.table).unwrap()).expect("Failed to write to file");
    }

    #[inline]
    pub fn get_table(&self) -> &toml::value::Table
    {
        return &self.table;
    }
}