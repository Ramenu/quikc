use std::{path::{PathBuf, Path}, fs::{File, self, Metadata}, time::UNIX_EPOCH, process::{Command}, sync::{atomic::{AtomicBool, Ordering}}, collections::HashSet};

use jwalk::WalkDir;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::{compiler::{self, INCLUDE_PATH_FLAG, INCLUDE_PATH}};


pub const BUILD_TABLE_DIRECTORY : &str = "./buildinfo";
pub const BUILD_TABLE_OBJECT_FILE_DIRECTORY : &str = "./buildinfo/obj";
pub const BUILD_TABLE_FILE : &str = "./buildinfo/table.toml";

pub struct BuildTable
{
    table : toml::value::Table,
    any_dependencies_changed : bool
}

#[inline]
fn get_duration_since_modified(metadata : &Metadata) -> i64
{
    return (metadata.modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_secs()) as i64;
}

fn file_modified_since_last_build(source_file_path : &mut PathBuf, 
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


impl BuildTable
{

    pub fn new(old_table : &mut toml::value::Table) -> BuildTable
    {

        // Create build object file directory
        if !Path::new(BUILD_TABLE_OBJECT_FILE_DIRECTORY).is_dir() {
            std::fs::create_dir(BUILD_TABLE_OBJECT_FILE_DIRECTORY).expect("Failed to create build object file directory");
        }

        if !Path::new(BUILD_TABLE_FILE).is_file() {
            File::create(BUILD_TABLE_FILE).expect("Failed to create file");
        }

        let file_contents = fs::read_to_string(BUILD_TABLE_FILE).expect("Failed to read from build table file");

        let mut table : toml::value::Table = toml::from_str(&file_contents).expect("Failed to parse build table file");
        let mut any_dependencies_changed = false;
        *old_table = table.clone();

        for path in WalkDir::new(INCLUDE_PATH) {
            let mut path = path.unwrap().path();
            let path_str = path.to_str().unwrap().to_string();

            let is_header_file = compiler::is_header_file(&path_str);
            // compiler doesnt show relative path sometimes so we need to address that
            let path_str_no_relative = if path_str.starts_with("./") { path_str[2..].to_string() } else { path_str };
            if is_header_file {
                let metadata = path.metadata().unwrap();
                let duration = get_duration_since_modified(&metadata);
                if file_modified_since_last_build(&mut path, 
                                                            &path_str_no_relative, 
                                                            true,
                                                            duration,
                                                                 &old_table) {
                    table.insert(path_str_no_relative, toml::Value::Integer(duration));
                    any_dependencies_changed = true;
                }
            }
        }

        return BuildTable {
            table,
            any_dependencies_changed
        };
    }

    fn get_file_dependencies(&self,
                             compiler_name : &str, 
                             source_file_name : &String) -> HashSet<String>
    {
        let cmd_output = &Command::new(compiler_name)
                                            .arg(INCLUDE_PATH_FLAG)
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
                                                        .collect::<HashSet<_>>();
                }
            }
            return HashSet::new();
        };
        return dependencies();

    }

    pub fn needs_to_be_recompiled(&mut self,
                                  source_file_path : &mut PathBuf,
                                  compiler_name : &str,
                                  old_table : &toml::value::Table) -> bool
    {
        let source_file_name = source_file_path.to_str().unwrap().to_string();


        // check if source file has changed (note we still need to check if any dependencies have changed to update
        // the build table)
        let source_modified_duration = get_duration_since_modified(&source_file_path.metadata().unwrap());
        let time = toml::Value::Integer(source_modified_duration);
        if file_modified_since_last_build(source_file_path, 
                                               &source_file_name, 
                                               false, 
                                               source_modified_duration,
                                               old_table) {
            self.table.insert(source_file_name, time);
            return true;
        }
        else if self.any_dependencies_changed {
            let recompile = AtomicBool::new(false);
            let dependencies = self.get_file_dependencies(compiler_name, &source_file_name);
            // check if any dependencies have changed for the source file 
            dependencies.par_iter().for_each(|dependency| {
                // sometimes the compiler shows '\' for line breaks, so we need to ignore those
                if dependency != "\\" {
                    let mut dependency_path = PathBuf::from(dependency);
                    let source_metadata = dependency_path.metadata().expect("Failed to retrieve metadata from file");
                    let duration = get_duration_since_modified(&source_metadata);
                    if file_modified_since_last_build(&mut dependency_path, 
                                                            dependency, 
                                                            true,
                                                            duration,
                                                                old_table) {
                        recompile.store(true, Ordering::Relaxed);
                        return;
                    }
                }
            });
            return recompile.load(Ordering::Relaxed);
        }
        return false;

    }


    #[inline]
    pub fn write(&self)
    {
        fs::write(BUILD_TABLE_FILE, toml::to_string(&self.table).unwrap()).expect("Failed to write to file");
    }

}