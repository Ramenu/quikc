use std::{process::Command, fs::{self}, env, path::Path, time::{SystemTime}};

use filetime::{set_file_mtime};

use crate::{build::{BUILD_CONFIG_FILE, Build}, SOURCE_DIRECTORY, compiler::{INCLUDE_PATH, compile_to_object_files, is_c_source_file, is_cpp_source_file, is_header_file}, buildtable::{BuildTable, BUILD_TABLE_OBJECT_FILE_DIRECTORY, get_duration_since_modified}, walker, linker::link_files};

const TOTAL_SOURCE_FILES : usize = 3;
const TEST_FILES_DIR : &str = "../testfiles";

pub struct Tools
{
    pub build_config : Build,
    pub source_files : Vec<String>,
    pub old_table : toml::value::Table,
    pub build_table : BuildTable
}

pub struct Settings
{
    pub use_clang : bool
}

impl Tools
{
    pub fn new() -> Tools
    {
        let build_config = Build::new();
        let mut old_table = toml::value::Table::new();
        let source_files = Vec::new();
        let build_table = BuildTable::new(&mut old_table);

        return Tools {
            build_config,
            source_files,
            old_table,
            build_table
        };
    }
}

#[inline]
fn get_source_file(file_name : &str) -> String
{
    return format!("{}/{}", SOURCE_DIRECTORY, file_name);
}

/// This function doesn't literally modify the file, but it
/// does change the time it was modified
pub fn modify_file_time(file : &str) -> Result<(), Box<dyn std::error::Error>>
{
    let time = get_duration_since_modified(&fs::metadata(file)?);
    set_file_mtime(file, SystemTime::now().into())?;
    let new_time = get_duration_since_modified(&fs::metadata(file)?);

    assert_ne!(time, new_time);

    Ok(())
}



#[inline]
fn get_src_files(tools : &mut Tools)
{
    walker::retrieve_source_files(SOURCE_DIRECTORY, 
                                  &mut tools.source_files, 
                                  &tools.build_config.get_compiler_name(), 
                                  &mut tools.build_table,
                                  &mut tools.old_table);
    
}

/// Initializes the project. If you the 'setup_additional_files' parameter is set to true,
/// then the function will copy additional source files to 'src'. If the 'with_invalid_file'
/// parameter is set to true, then the function will copy an invalid source file to 'src'.
/// This should be done if you want to check if quikc will recompile the source file after
/// the error.
pub fn initialize_project(setup_additional_files : bool, 
                      with_invalid_file : bool,
                      settings : &Settings) -> Result<(), Box<dyn std::error::Error>>
{
    const INVALID_FILE_NAME : &str = "invalid.c";
    let invalid_file = format!("{}/invalid/{}", TEST_FILES_DIR, INVALID_FILE_NAME);

    to_test_directory()?;

    const TEST_PACKAGE_NAME : &str = "test_binary";
    let status = Command::new("python")
                                     .arg("../quikc-init")
                                     .arg(TEST_PACKAGE_NAME)
                                     .spawn()?
                                     .wait()?;
    assert_eq!(status.success(), true);

    if settings.use_clang {
        let clang_build_toml = format!("{TEST_FILES_DIR}/config/{BUILD_CONFIG_FILE}");
        fs::copy(clang_build_toml, BUILD_CONFIG_FILE)?;
    }

    if setup_additional_files {
        let dir = fs::read_dir(TEST_FILES_DIR)?;
        for entry in dir {
            let entry = entry?;
            let path = entry.path();
            let name = path.as_os_str().to_str().unwrap();
            if path.is_file() {
                if is_c_source_file(name) || is_cpp_source_file(name){
                    fs::copy(name, format!("{}/{}", SOURCE_DIRECTORY, path.file_name().unwrap().to_str().unwrap()))?;
                }
                else if is_header_file(name) {
                    fs::copy(name, format!("{}/{}", INCLUDE_PATH, path.file_name().unwrap().to_str().unwrap()))?;
                }
            }
        }

        if with_invalid_file {
            fs::copy(invalid_file, format!("{}/{}", SOURCE_DIRECTORY, INVALID_FILE_NAME))?;
        }
    }

    Ok(())
}

fn to_test_directory() -> Result<(), Box<dyn std::error::Error>>
{
    const TEST_DIR : &str = "./testdir";
    // Remove contents from test directory if it existed already
    if Path::new(TEST_DIR).is_dir() {
        fs::remove_dir_all(TEST_DIR)?;
    }
    fs::create_dir(TEST_DIR)?;
    env::set_current_dir("./testdir")?;
    Ok(())
}

fn reset() -> Result<(), Box<dyn std::error::Error>>
{
    env::set_current_dir("..")?;
    fs::remove_dir_all("./testdir")?;
    Ok(())
}

/// This is meant to be used to test all of the functions.
/// It runs it sequentially because running them independently
/// can have unintended side affects.
#[test]
fn test_all() -> Result<(), Box<dyn std::error::Error>>
{
    let mut settings = Settings{use_clang : false};
    for _ in 0..2 {
        test_quikc_init(&settings)?;
        reset()?;

        test_first_time_compilation(&settings)?;
        reset()?;

        test_recompilation(&settings)?;
        reset()?;

        test_invalid_file_recompiles(&settings)?;
        reset()?;

        test_recompile_after_config_change(&settings)?;
        reset()?;

        test_recompile_after_deletion(&settings)?;
        reset()?;

        settings.use_clang = true;
    }

    Ok(())
}

fn test_quikc_init(settings : &Settings) ->  Result<(), Box<dyn std::error::Error>>
{


    // 'initialize_project' will create many source files, however the file generated
    // by the 'quikc-init' command is the only one we need to check for. The other ones
    // are for testing purposes only, which is why only 'source_file' is checked
    initialize_project(false, false, settings)?;

    let source_file = format!("{}/main.c", SOURCE_DIRECTORY);

    assert_eq!(Path::new(BUILD_CONFIG_FILE).is_file(), true);
    assert_eq!(Path::new(SOURCE_DIRECTORY).is_dir(), true);
    assert_eq!(Path::new(INCLUDE_PATH).is_dir(), true);
    assert_eq!(Path::new(&source_file).is_file(), true);
    
    Ok(())
}

/// This will treat the project as if it needs to be rebuilt entirely.
fn test_first_time_compilation(settings : &Settings) -> Result<(), Box<dyn std::error::Error>>
{
    initialize_project(true, false, settings)?;
    let mut tools = Tools::new();
    get_src_files(&mut tools);

    assert_eq!(tools.source_files.len(), TOTAL_SOURCE_FILES);

    let compilation_success = compile_to_object_files(&mut tools.source_files, &tools.build_config);
    assert_eq!(compilation_success, true);
    assert_eq!(fs::read_dir(BUILD_TABLE_OBJECT_FILE_DIRECTORY)?.count(), TOTAL_SOURCE_FILES);

    let link_success = link_files(&tools.build_config);
    assert_eq!(link_success, true);


    Ok(())
}

/// Will test if the files will recompile after being modified.
/// This includes header files and source files.
fn test_recompilation(settings : &Settings) -> Result<(), Box<dyn std::error::Error>>
{
    test_first_time_compilation(settings)?; 

    // note we introduce different scopes so the build table file is written to
    // once the build table has been dropped

    // Compiled it once, now we modify a specific source file and recompile
    {
        let source_file_to_modify = format!("{}/{}", SOURCE_DIRECTORY, "main.c");
        modify_file_time(source_file_to_modify.as_str())?;
        let mut tools = Tools::new();
        get_src_files(&mut tools);

        // Should be only 1 file that was added, since we modified one file only
        assert_eq!(tools.source_files.len(), 1); 
        let compilation_success = compile_to_object_files(&mut tools.source_files, &tools.build_config);

        assert_eq!(compilation_success, true);
        assert_eq!(fs::read_dir(BUILD_TABLE_OBJECT_FILE_DIRECTORY)?.count(), TOTAL_SOURCE_FILES);

        let link_success = link_files(&tools.build_config);
        assert_eq!(link_success, true);
    }

    // Modify the header file, once modified, then all of the source files that
    // depended on it need to be recompiled
    {
        let header_file_to_modify = format!("{}/{}", INCLUDE_PATH, "hi.h");
        modify_file_time(header_file_to_modify.as_str())?;
        let mut tools = Tools::new();
        get_src_files(&mut tools);

        // 3 source files depend on the header
        assert_eq!(tools.source_files.len(), 3);

        let compilation_success = compile_to_object_files(&mut tools.source_files, &tools.build_config);
        assert_eq!(compilation_success, true);
        assert_eq!(fs::read_dir(BUILD_TABLE_OBJECT_FILE_DIRECTORY)?.count(), TOTAL_SOURCE_FILES);

        let link_success = link_files(&tools.build_config);
        assert_eq!(link_success, true);
    }

    Ok(())
}

/// Will test if 'quikc' knows to recompile a file again if it
/// had an error.
fn test_invalid_file_recompiles(settings : &Settings) -> Result<(), Box<dyn std::error::Error>>
{
    {
        initialize_project(true, true, settings)?;

        // TOTAL_SOURCE_FILES + 1 because we added an invalid file
        const TOTAL_FILES : usize = TOTAL_SOURCE_FILES + 1;

        let mut tools = Tools::new();
        get_src_files(&mut tools);

        assert_eq!(tools.source_files.len(), TOTAL_FILES);
        let compilation_success = compile_to_object_files(&mut tools.source_files, &tools.build_config);

        // Compilation should have failed since the invalid file has a error in it
        assert_eq!(compilation_success, false); 

        // There should only be 'TOTAL_FILES - 1' object files since the invalid file
        // did not compile successfully
        assert_eq!(fs::read_dir(BUILD_TABLE_OBJECT_FILE_DIRECTORY)?.count(), TOTAL_FILES - 1);
    }

    // Now we compile again
    let mut tools = Tools::new();
    get_src_files(&mut tools);

    // The invalid file should have been the only file that needed to be recompiled
    assert_eq!(tools.source_files.len(), 1);

    Ok(())
}

/// Tests if the entire project will recompile if the build config file has been changed.
fn test_recompile_after_config_change(settings : &Settings) -> Result<(), Box<dyn std::error::Error>>
{
    test_first_time_compilation(settings)?;

    let build_config_file_new = format!("{}/{}", TEST_FILES_DIR, BUILD_CONFIG_FILE);
    fs::copy(build_config_file_new, BUILD_CONFIG_FILE)?;

    let mut tools = Tools::new();
    get_src_files(&mut tools);

    // All of the source files should be recompiled since the build config file has been changed
    assert_eq!(tools.source_files.len(), TOTAL_SOURCE_FILES);
    let compilation_success = compile_to_object_files(&mut tools.source_files, &tools.build_config);

    assert_eq!(compilation_success, true);
    assert_eq!(fs::read_dir(BUILD_TABLE_OBJECT_FILE_DIRECTORY)?.count(), TOTAL_SOURCE_FILES);

    let link_success = link_files(&tools.build_config);
    assert_eq!(link_success, true);

    Ok(())
}

/// Tests if the entire project will recompile if a source file has been deleted.
fn test_recompile_after_deletion(settings : &Settings) -> Result<(), Box<dyn std::error::Error>>
{
    const FILE_TO_BE_DELETED : &str = "dep.c";
    test_first_time_compilation(settings)?;

    fs::remove_file(get_source_file(FILE_TO_BE_DELETED))?;
    
    const NUM_FILES_AFTER_DELETION : usize = TOTAL_SOURCE_FILES - 1;
    // Once the file is removed, recompilation should begin
    {
        let mut tools = Tools::new();
        get_src_files(&mut tools);

        assert_eq!(tools.source_files.len(), NUM_FILES_AFTER_DELETION);

        let compilation_success = compile_to_object_files(&mut tools.source_files, &tools.build_config);

        assert_eq!(compilation_success, true);
        assert_eq!(fs::read_dir(BUILD_TABLE_OBJECT_FILE_DIRECTORY)?.count(), NUM_FILES_AFTER_DELETION);
        assert_eq!(tools.build_table.contains(format!("{}/{}", SOURCE_DIRECTORY, FILE_TO_BE_DELETED).as_str()), false);

        let link_success = link_files(&tools.build_config);
        assert_eq!(link_success, true);

    }

    Ok(())
}