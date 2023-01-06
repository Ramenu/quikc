use std::{path::{PathBuf, Path}, fs::{File, OpenOptions, self, Metadata}, io::Write, time::UNIX_EPOCH};

use toml::value::Datetime;

use crate::{build, compiler};



pub struct BuildTable
{
    build_table_file : String,
    table : toml::value::Table
}

impl BuildTable
{

    pub fn new(build_dir : &'static str) -> BuildTable
    {
        if !Path::new(build_dir).is_dir() {
            std::fs::create_dir(build_dir).expect("Failed to create directory");
        }

        let build_table_file = format!("{}/{}", build_dir, "table.toml");

        if !Path::new(&build_table_file).is_file() {
            File::create(&build_table_file).expect("Failed to create file");
        }

        let file_contents = fs::read_to_string(&build_table_file).expect("Failed to read from build table file");

        let table : toml::value::Table = toml::from_str(&file_contents).expect("Failed to parse build table file");

        return BuildTable {
            build_table_file,
            table
        };
    }


    pub fn needs_to_be_recompiled(&mut self, file_path : &PathBuf, object_file_dir : &str) -> bool
    {
        // Elapsed time since the source file was edited in the table
        let file_name = file_path.to_str().unwrap().to_string();

        // retrieve source file's metadata
        let file_path_metadata = file_path.metadata().expect("Failed to retrieve metadata from file");

        // retrieve the time that has been elapsed since it was last modified (in seconds)
        let time = (file_path_metadata.modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_secs() / 6000) as i64;

        // check if value exists in the table, if so, compare the times,
        // if they are the same, then the source file has not been modified,
        // so recompilation is not necessary. Otherwise, it is
        if self.table.contains_key(&file_name) {
            let old_value = self.table.get(&file_name).unwrap().as_integer().unwrap();
            println!("Previous time: {}\nCurrent time: {}", old_value, time);
            if old_value != time {
                self.table.insert(file_name, toml::Value::Integer(time));
                return true;
            }

            let object_file = compiler::to_object_file(&mut file_path.to_owned(), object_file_dir);
            
            println!("{}", &object_file);
            // If the object file does exist, compilation was most likely successful, if not
            // then re-compilation is necessary
            if Path::new(&object_file).exists() {
                return false;
            }
        }

        self.table.insert(file_name, toml::Value::Integer(time));
        return true;
    }


    #[inline]
    pub fn write(&self)
    {
        fs::write(&self.build_table_file, toml::to_string(&self.table).unwrap()).expect("Failed to write to file");
    }
}