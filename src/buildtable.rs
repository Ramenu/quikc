use std::{path::{PathBuf, Path}, fs::{File, self, Metadata}, time::UNIX_EPOCH, sync::{atomic::{AtomicBool, Ordering}}, collections::{HashSet, HashMap}, io::Write};

use const_format::concatcp;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use walkdir::WalkDir;

use crate::{compiler::{self, INCLUDE_PATH}, flags, QuikcFlags};
use bitflags::bitflags;

pub const BUILD_TABLE_DIRECTORY : &str = "./buildinfo";
pub const BUILD_TABLE_OBJECT_FILE_DIRECTORY : &str = concatcp!(BUILD_TABLE_DIRECTORY, "/obj");
pub const BUILD_TABLE_FILE : &str = concatcp!(BUILD_TABLE_DIRECTORY, "/table");
pub const BUILD_TABLE_DEPS_DIRECTORY : &str = concatcp!(BUILD_TABLE_DIRECTORY, "/deps");
pub const BUILD_TABLE_ASM_DIRECTORY : &str = concatcp!(BUILD_TABLE_DIRECTORY, "/asm");

bitflags! {
    struct BuildTableFlags : u8 {
        const NONE = 0;
        const ANY_DEPENDENCIES_CHANGED = 1 << 0;
    }
}

pub struct BuildTable
{
    table : HashMap<String, u64>,
    flags : BuildTableFlags
}

#[inline]
pub fn get_duration_since_modified(metadata : &Metadata) -> u64
{
    (metadata.modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_millis()) as u64
}

/// Returns true if the file has been modified since the last build.
fn file_modified_since_last_build(source_file_path : &Path, 
                                  source_file_name : &str,
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
        
        // only check for object/assembly file if the source isnt a header
        if !is_header_file {

            // If the user wants an assembly output, check if an assembly version already exists
            // and if it does, then recompilation is not necessary
            if flags()&QuikcFlags::ASSEMBLE == QuikcFlags::ASSEMBLE {
                let assembly_file = compiler::to_output_file(source_file_path, BUILD_TABLE_ASM_DIRECTORY, "s");
                if Path::new(&assembly_file).exists() {
                    return false;
                }
            }

            // If the object file does exist, compilation was most likely successful, if not
            // then re-compilation is necessary
            else {
                let object_file = compiler::to_output_file(source_file_path, BUILD_TABLE_OBJECT_FILE_DIRECTORY, "o");
                
                if Path::new(&object_file).exists() {
                    return false;
                }
            }
            return true;
        }
        return false;
    }

    true
}


impl BuildTable
{
    /// Creates a new build table. This should only be created once
    /// at the beginning of the program as creating it is expensive.
    /// It will initialize the old table's state to the contents in
    /// '/buildinfo/table'. This is done so that the current table
    /// can be modified and compared to the old table.
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
            File::create(BUILD_TABLE_FILE).expect("Failed to create build table file");
        }

        let file_contents = fs::read_to_string(BUILD_TABLE_FILE).expect("Failed to read file");
        let mut table = HashMap::new();
        let mut flags = BuildTableFlags::NONE;
        
        // If the file is empty, then we do not have to check if any modifications were 
        // made and can simply just add the all header files to the table
        if !file_contents.is_empty() {
            for line in file_contents.lines() {
                let split : Vec<&str> = line.split('=').collect();
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
                    flags |= BuildTableFlags::ANY_DEPENDENCIES_CHANGED;
                }
            }
        
            // recursively walk through the include directory
            for path in WalkDir::new(INCLUDE_PATH) {
                let path = path.unwrap().path().to_path_buf();
                if path.is_file() {
                    let path_str = path.to_str().unwrap();
                    let is_header_file = compiler::is_header_file(path_str);

                    if is_header_file {
                        // compiler doesnt show relative path sometimes so we need to address that
                        let path_str_no_relative = if let Some(stripped) = path_str.strip_prefix("./") 
                            { stripped } else { path_str };
                        
                        // compare the header's file current modification time with the time set in the table,
                        // if the current time is newer, then we can insert the new time into the table and
                        // mark 'any_dependencies_changed' as true
                        let metadata = path.metadata().unwrap();
                        let duration = get_duration_since_modified(&metadata);
                        if file_modified_since_last_build(&path, 
                                                                path_str_no_relative, 
                                                                    true,
                                                                    duration,
                                                                        old_table) {
                            table.insert(path_str_no_relative.to_string(), duration);
                            flags |= BuildTableFlags::ANY_DEPENDENCIES_CHANGED;
                        }
                    }
                }
            }
        }
        else {
            for path in WalkDir::new(INCLUDE_PATH) {
                let path = path.unwrap().path().to_path_buf();

                if path.is_file() {
                    let path_str = path.to_str().unwrap();
                    let is_header_file = compiler::is_header_file(path_str);
                    if is_header_file {
                        // compiler doesnt show relative path sometimes so we need to address that
                        let path_str_no_relative = if let Some(stripped) = path_str.strip_prefix("./") 
                            { stripped } else { path_str };

                        // since the build table is empty, there is nothing to compare to, so we just insert
                        // the file
                        let metadata = path.metadata().unwrap();
                        let duration = get_duration_since_modified(&metadata);
                        table.insert(path_str_no_relative.to_string(), duration);
                        flags |= BuildTableFlags::ANY_DEPENDENCIES_CHANGED;
                    }
                }
            }
        }

        BuildTable {
            table,
            flags
        }
    }

    /// Returns the given source file's dependencies as a hashset (to avoid duplicates). 
    /// Note that this doesn't include system header files as they very rarely change
    /// often. Future versions may include a flag to count system dependencies as well.
    pub fn get_file_dependencies(&self, source_file_name : &str) -> HashSet<String>
    {
        let path = Path::new(source_file_name);
        let file_name = path.file_stem().unwrap().to_str().unwrap();
        let dep_name = BUILD_TABLE_DEPS_DIRECTORY.to_string() + "/" + file_name + ".d";

        if !Path::new(&dep_name).is_file() {
            return HashSet::new();
        }

        let dependencies_str_tmp = fs::read_to_string(&dep_name).unwrap();
        
        let dependencies = move || {
            let mut count = 0;
            // at least for clang and gcc, they output the dependencies in a format like:
            // source.o: source.c <dependencies>
            // This follows the make format. As you can see from this, we need to skip the
            // first two spaces to get the actual list of dependencies.
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
            HashSet::new()
        };
        dependencies()

    }

    /// Returns true if 'source_file_path' needs to be recompiled.
    /// Header files do not count as a source file, only files with
    /// a .c, .cpp, .cxx, .cc extension count. 
    pub fn needs_to_be_recompiled(&mut self,
                                  source_file_path : &Path,
                                  old_table : &HashMap<String, u64>) -> bool
    {
        let source_file_name = source_file_path.to_str().unwrap();


        // check if source file has changed, if so, we don't need to check if any of the dependencies
        // changed which is best case scenario.
        let source_modified_duration = get_duration_since_modified(&source_file_path.metadata().unwrap());
        if file_modified_since_last_build(source_file_path, 
                                               source_file_name, 
                                               false, 
                                               source_modified_duration,
                                               old_table) {
            // insert the new modification time
            self.table.insert(source_file_name.to_string(), source_modified_duration);
            return true;
        }
        else if self.flags&BuildTableFlags::ANY_DEPENDENCIES_CHANGED == BuildTableFlags::ANY_DEPENDENCIES_CHANGED {
            let dependencies = self.get_file_dependencies(source_file_name);
            const LARGE_NUMBER_OF_FILES : usize = 50;

            // Doing this in parallel can actually be significantly slower if there aren't a lot of
            // dependencies so it important to check if it necessary
            if dependencies.len() >= LARGE_NUMBER_OF_FILES {
                let recompile = AtomicBool::new(false);
                // check if any dependencies have changed for the source file 
                dependencies.par_iter().for_each(|dependency| {
                    // sometimes the compiler shows '\' for line breaks, so we need to ignore those
                    if dependency != "\\" {
                        let dependency_path = PathBuf::from(dependency);
                        // some dependencies may have been deleted or moved to different locations
                        // since last compilation so its important to check if it exists first
                        if dependency_path.exists() {
                            let source_metadata = dependency_path.metadata().expect("Failed to retrieve metadata from file");
                            let duration = get_duration_since_modified(&source_metadata);
                            if file_modified_since_last_build(&dependency_path, 
                                                                    dependency, 
                                                                    true,
                                                                    duration,
                                                                        old_table) {
                                recompile.store(true, Ordering::Relaxed);
                                
                            }
                        }
                        else {
                            // dependency was deleted or moved so we need to recompile 
                            // the file
                            recompile.store(true, Ordering::Relaxed);
                            
                        }
                    }
                });
                return recompile.load(Ordering::Relaxed);
            }

            for dependency in dependencies {
                if dependency != "\\" {
                    let dependency_path = PathBuf::from(&dependency);
                    if dependency_path.exists() {
                        let source_metadata = dependency_path.metadata().expect("Failed to retrieve metadata from file");
                        let duration = get_duration_since_modified(&source_metadata);
                        if file_modified_since_last_build(&dependency_path, 
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
        false

    }

    /// Removes `path_str` from the build table.
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
        if source_files_changed {
            self.flags |= BuildTableFlags::ANY_DEPENDENCIES_CHANGED;
        }
        else {
            self.flags ^= BuildTableFlags::ANY_DEPENDENCIES_CHANGED;
        }
    }

    /// Returns true if the build table contains `path_str`.
    #[cfg(test)]
    pub fn contains(&self, path_str : &str) -> bool
    {
        self.table.contains_key(path_str)
    }

}

impl Drop for BuildTable
{
    /// I doubt this is a good idea....
    fn drop(&mut self)
    {
        // No point of writing to file if none of the dependencies changed, and writing to file must be
        // explicitly enabled
        if self.flags&BuildTableFlags::ANY_DEPENDENCIES_CHANGED == BuildTableFlags::ANY_DEPENDENCIES_CHANGED {
            let mut f = File::create(BUILD_TABLE_FILE).expect("Failed to create build table file");
            for (k, v) in &self.table {
                f.write_all(format!("{k}={v}\n").as_bytes()).expect("Failed to write to build table file");
            }
        }
    }
}