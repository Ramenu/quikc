use std::{process::Command, fs, env, path::Path};

use crate::{build::BUILD_CONFIG_FILE, SOURCE_DIRECTORY, compiler::INCLUDE_PATH};




#[test]
fn test_quikc_init() ->  Result<(), Box<dyn std::error::Error>>
{
    const TEST_DIR : &str = "./testdir";
    const TEST_PACKAGE_NAME : &str = "test_binary";

    let source_file = format!("{}/main.c", SOURCE_DIRECTORY);

    // Remove contents from test directory if it existed already
    if Path::new(TEST_DIR).is_dir() {
        fs::remove_dir_all(TEST_DIR)?; 
    }

    fs::create_dir(TEST_DIR)?;
    env::set_current_dir(TEST_DIR)?;
    let status = Command::new("python").arg("../quikc-init").arg(TEST_PACKAGE_NAME).spawn()?.wait()?;

    assert_eq!(status.success(), true);
    assert_eq!(Path::new(BUILD_CONFIG_FILE).is_file(), true);
    assert_eq!(Path::new(SOURCE_DIRECTORY).is_dir(), true);
    assert_eq!(Path::new(INCLUDE_PATH).is_dir(), true);
    assert_eq!(Path::new(&source_file).is_file(), true);
    
    Ok(())
}