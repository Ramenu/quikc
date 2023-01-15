use std::{process::Command, fs, env, path::Path, io};

use crate::{build::BUILD_CONFIG_FILE, SOURCE_DIRECTORY, compiler::INCLUDE_PATH};

/// Initializes the project. If you the 'setup_additional_files' parameter is set to true,
/// then the function will copy additional source files to 'src'.
fn initialize_project(setup_additional_files : bool) -> Result<(), Box<dyn std::error::Error>>
{
    to_test_directory()?;

    const TEST_PACKAGE_NAME : &str = "test_binary";
    let status = Command::new("python")
                                     .arg("../quikc-init")
                                     .arg(TEST_PACKAGE_NAME)
                                     .spawn()?
                                     .wait()?;
    assert_eq!(status.success(), true);

    if setup_additional_files {
        let dir = fs::read_dir("../testfiles")?;
        for entry in dir {
            let entry = entry?;
            let path = entry.path();
            let name = path.as_os_str().to_str().unwrap();
            if path.is_file() {
                fs::copy(name, format!("{}/{}", SOURCE_DIRECTORY, path.file_name().unwrap().to_str().unwrap()))?;
            }
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


#[test]
fn test_quikc_init() ->  Result<(), Box<dyn std::error::Error>>
{


    // 'initialize_project' will create many source files, however the file generated
    // by the 'quikc-init' command is the only one we need to check for. The other ones
    // are for testing purposes only, which is why only 'source_file' is checked
    initialize_project(false)?;

    let source_file = format!("{}/main.c", SOURCE_DIRECTORY);

    assert_eq!(Path::new(BUILD_CONFIG_FILE).is_file(), true);
    assert_eq!(Path::new(SOURCE_DIRECTORY).is_dir(), true);
    assert_eq!(Path::new(INCLUDE_PATH).is_dir(), true);
    assert_eq!(Path::new(&source_file).is_file(), true);
    
    Ok(())
}

/// This will treat the project as if it needs to be rebuilt entirely.
#[test]
fn test_first_time_compilation() -> Result<(), Box<dyn std::error::Error>>
{
    initialize_project(true)?;
    

    Ok(())

}