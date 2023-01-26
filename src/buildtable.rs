use std::{path::{PathBuf, Path}, fs::{File, self, Metadata}, time::UNIX_EPOCH, sync::{atomic::{AtomicBool, Ordering}}, collections::{HashSet, HashMap}, io::Write};

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use walkdir::WalkDir;

use crate::{compiler::{self, INCLUDE_PATH}};


pub const BUILD_TABLE_DIRECTORY : &str = "./buildinfo";
pub const BUILD_TABLE_OBJECT_FILE_DIRECTORY : &str = "./buildinfo/obj";
pub const BUILD_TABLE_FILE : &str = "./buildinfo/table";
pub const BUILD_TABLE_DEPS_DIRECTORY : &str = "./buildinfo/deps";

pub struct BuildTable
{
    table : HashMap<String, u64>,
    any_dependencies_changed : bool
}

#[inline]
pub fn get_duration_since_modified(metadata : &Metadata) -> u64
{
    return (metadata.modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_millis()) as u64;
}

fn file_modified_since_last_build(source_file_path : &mut PathBuf, 
                                  source_file_name : &String,
                                  is_header_file : bool,
                                  time : u64,
                                  old_table : &HashMap<String, u64>) -> bool
{

    // check if value exists in the table, if so, compare the times,
    // if they are the same, then the source file has not been modified,
    // so recompilation is not necessary. Otherwise, it is
    if old_table.contains_key(source_file_name) {
        let old_value = old_table[source_file_name];

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

    pub fn new(old_table : &mut HashMap<String, u64>) -> BuildTable
    {

        // Create build object file directory
        if !Path::new(BUILD_TABLE_OBJECT_FILE_DIRECTORY).is_dir() {
            std::fs::create_dir(BUILD_TABLE_OBJECT_FILE_DIRECTORY).expect("Failed to create build object file directory");
        }

        if !Path::new(BUILD_TABLE_DEPS_DIRECTORY).is_dir() {
            std::fs::create_dir(BUILD_TABLE_DEPS_DIRECTORY).expect("Failed to create build dependencies directory");
        }

        if !Path::new(BUILD_TABLE_FILE).is_file() {
            File::create(BUILD_TABLE_FILE).expect("Failed to create file");
        }

        let file_contents = fs::read_to_string(BUILD_TABLE_FILE).expect("Failed to read file");
        let mut table = HashMap::new();
        let mut any_dependencies_changed = false;
        
        // If the file is empty, then we do not have to check if any modifications were 
        // made and can simply just add the all header files to the table
        if !file_contents.is_empty() {
            for line in file_contents.lines() {
                let split : Vec<&str> = line.split("=").collect();
                let key = split[0].to_string();
                let value = split[1].parse::<u64>().unwrap();
                
                let exists = Path::new(&key).exists();

                // It's important to check if the file exists here,
                // dependencies may have gotten deleted or moved somewhere
                // else since the last build. If this is the case, we need
                // to confirm that the file is in fact a dependency, and if so,
                // mark 'any_dependencies_changed' as true.
                if exists {
                    table.insert(key.to_owned(), value);
                    old_table.insert(key, value);
                }
                else if compiler::is_header_file(&key){
                    any_dependencies_changed = true;
                }
            }
        

            for path in WalkDir::new(INCLUDE_PATH) {
                let mut path = path.unwrap().path().to_path_buf();
                if path.is_file() {
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
                            table.insert(path_str_no_relative, duration);
                            any_dependencies_changed = true;
                        }
                    }
                }
            }
        }
        else {
            for path in WalkDir::new(INCLUDE_PATH) {
                let path = path.unwrap().path().to_path_buf();

                if path.is_file() {
                    let path_str = path.to_str().unwrap().to_string();
                    let is_header_file = compiler::is_header_file(&path_str);
                    // compiler doesnt show relative path sometimes so we need to address that
                    let path_str_no_relative = if path_str.starts_with("./") { path_str[2..].to_string() } else { path_str };
                    if is_header_file {
                        let metadata = path.metadata().unwrap();
                        let duration = get_duration_since_modified(&metadata);
                        table.insert(path_str_no_relative, duration);
                    }
                }
            }
            any_dependencies_changed = true;
        }

        return BuildTable {
            table,
            any_dependencies_changed
        };
    }

    pub fn get_file_dependencies(&self, source_file_name : &str) -> HashSet<String>
    {
        let mut path = PathBuf::from(source_file_name);
        path.set_extension("");
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let dep_name = BUILD_TABLE_DEPS_DIRECTORY.to_string() + "/" + file_name + ".d";

        if !Path::new(&dep_name).is_file() {
            return HashSet::new();
        }

        let dependencies_str_tmp = fs::read_to_string(&dep_name).unwrap();
        
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
                                  old_table : &HashMap<String, u64>) -> bool
    {
        let source_file_name = source_file_path.to_str().unwrap().to_string();


        // check if source file has changed (note we still need to check if any dependencies have changed to update
        // the build table)
        let source_modified_duration = get_duration_since_modified(&source_file_path.metadata().unwrap()) as u64;
        if file_modified_since_last_build(source_file_path, 
                                               &source_file_name, 
                                               false, 
                                               source_modified_duration,
                                               old_table) {
            self.table.insert(source_file_name, source_modified_duration);
            return true;
        }
        else if self.any_dependencies_changed {
            let dependencies = self.get_file_dependencies(&source_file_name);
            const LARGE_NUMBER_OF_FILES : usize = 50;

            // Doing this in parallel can actually be significantly slower if there aren't a lot of
            // dependencies so it important to check if it necessary
            if dependencies.len() >= LARGE_NUMBER_OF_FILES {
                let recompile = AtomicBool::new(false);
                // check if any dependencies have changed for the source file 
                dependencies.par_iter().for_each(|dependency| {
                    // sometimes the compiler shows '\' for line breaks, so we need to ignore those
                    if dependency != "\\" {
                        let mut dependency_path = PathBuf::from(dependency);
                        // some dependencies may have been deleted or moved to different locations
                        // since last compilation so its important to check if it exists first
                        if dependency_path.exists() {
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
                        else {
                            // dependency was deleted or moved so we need to recompile 
                            // the file
                            recompile.store(true, Ordering::Relaxed);
                            return;
                        }
                    }
                });
                return recompile.load(Ordering::Relaxed);
            }

            for dependency in dependencies {
                if dependency != "\\" {
                    let mut dependency_path = PathBuf::from(&dependency);
                    if dependency_path.exists() {
                        let source_metadata = dependency_path.metadata().expect("Failed to retrieve metadata from file");
                        let duration = get_duration_since_modified(&source_metadata);
                        if file_modified_since_last_build(&mut dependency_path, 
                                                                &dependency, 
                                                                true,
                                                                duration,
                                                                    old_table) {
                            return true;
                        }
                    }
                    else { 
                        return true;
                    }
                }
            }
        }
        return false;

    }

    #[inline]
    pub fn erase(&mut self, path_str : &str)
    {
        self.table.remove(path_str);
    }

    /// This should only be called after you check if every source file
    /// has changed. 
    #[inline]
    pub fn set_any_dependencies_changed(&mut self, source_files_changed : bool) 
    {
        self.any_dependencies_changed |= source_files_changed;
    }

    #[cfg(test)]
    pub fn contains(&self, path_str : &str) -> bool
    {
        return self.table.contains_key(path_str);
    }

}

impl Drop for BuildTable
{
    #[inline]
    fn drop(&mut self)
    {
        // No point of writing to file if none of the dependencies changed
        if self.any_dependencies_changed {
            let mut f = File::create(BUILD_TABLE_FILE).expect("Failed to create build table file");
            for (k, v) in &self.table {
                f.write(format!("{}={}\n", k, v).as_bytes()).expect("Failed to write to build table file");
            }
        }
    }
}