use std::{process::Command, fs::{self}, env, path::Path, time::{SystemTime}, collections::HashMap, io::Write};
#[allow(unused_imports)]
use crate::{defaultbuild::{GCC_AND_CLANG_LINKER_OPTIONS, GCC_COMPILER_C_EXCLUSIVE_WARNINGS, GCC_COMPILER_NONEXCLUSIVE_WARNINGS, GCC_AND_CLANG_DIALECT_OPTIONS, CLANG_COMPILER_NONEXCLUSIVE_WARNINGS, GCC_COMPILER_CPP_EXCLUSIVE_WARNINGS, GCC_COMPILER_CPP_DIALECT_OPTIONS, CLANG_COMPILER_CPP_WARNINGS, GCC_AND_CLANG_CPP_DIALECT_OPTIONS, GCC_AND_CLANG_OPTIMIZATION_OPTIONS, GCC_STATIC_ANALYSIS_OPTIONS, GCC_AND_CLANG_ENHANCED_OPTIMIZATION_OPTIONS}, build::{DEFAULT_C_STANDARD, DEFAULT_CPP_STANDARD}, version::VERSIONS};

use color_print::cprintln;
use const_format::concatcp;
use filetime::{set_file_mtime};
#[cfg(feature = "quikc-nightly")]
use crate::version::NIGHTLY_VERSION;

use crate::{build::{BUILD_CONFIG_FILE, Build}, SOURCE_DIRECTORY, compiler::{INCLUDE_PATH, compile_to_object_files, is_c_source_file, is_cpp_source_file, is_header_file}, buildtable::{BuildTable, BUILD_TABLE_OBJECT_FILE_DIRECTORY, get_duration_since_modified, BUILD_TABLE_ASM_DIRECTORY}, walker, linker::link_files, set_flags};

const TOTAL_SOURCE_FILES : usize = 3;
const TEST_FILES_DIR : &str = "../testfiles";
const TEST_PACKAGE_NAME : &str = "main";

pub struct Tools
{
    pub build_config : Build,
    pub source_files : Vec<String>,
    pub old_table : HashMap<String, u64>,
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
        let mut old_table = HashMap::new();
        let source_files = Vec::new();
        let build_table = BuildTable::new(&mut old_table);

        Tools {
            build_config,
            source_files,
            old_table,
            build_table
        }
    }
}

#[inline]
fn get_source_file(file_name : &str) -> String
{
    format!("{SOURCE_DIRECTORY}/{file_name}")
}

#[inline]
fn get_dependency_file(file_name : &str) -> String
{
    format!("{INCLUDE_PATH}/{file_name}")
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

fn write_to_config(build_config : &Build) -> Result<(), Box<dyn std::error::Error>>
{
    let mut file = fs::File::create(BUILD_CONFIG_FILE)?;
    let toml = toml::to_string(build_config)?;
    file.write_all(toml.as_bytes())?;
    Ok(())
}


#[inline]
fn get_src_files(tools : &mut Tools)
{
    (tools.source_files, _, _) = walker::retrieve_source_files(SOURCE_DIRECTORY, 
                                                            &mut tools.build_table,
                                                            &tools.old_table);
    
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
    let invalid_file = format!("{TEST_FILES_DIR}/invalid/{INVALID_FILE_NAME}");

    to_test_directory()?;

    let status = Command::new("python")
                                     .arg("../quikc-init")
                                     .arg(TEST_PACKAGE_NAME)
                                     .spawn()?
                                     .wait()?;
    assert!(status.success());

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
            fs::copy(invalid_file, format!("{SOURCE_DIRECTORY}/{INVALID_FILE_NAME}"))?;
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

/// Deletes everything in 'testdir' and sets the current directory to the parent directory
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
pub fn test_all() -> Result<(), Box<dyn std::error::Error>>
{
    set_flags();
    let mut settings = Settings{use_clang : false};

    // run it 2 times using GCC and clang
    for _ in 0..2 {
        test_quikc_init(&settings)?;
        reset()?;

        test_config(&settings)?;
        reset()?;

        test_cmdline_flags(&settings)?;
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

        test_recompilation_after_deleting_binary(&settings)?;
        reset()?;

        test_compilation_after_dependency_deletion(&settings)?;
        reset()?;

        test_execute_compiler_with_build_info(&settings)?;
        reset()?;

        test_execute_linker_with_build_info(&settings)?;
        reset()?;

        settings.use_clang = true;
    }

    cprintln!("<bold> {}-- <green>All tests passed</green> --{}</bold>", '<', '>');
    Ok(())
}

fn test_quikc_init(settings : &Settings) ->  Result<(), Box<dyn std::error::Error>>
{


    // 'initialize_project' will create many source files, however the file generated
    // by the 'quikc-init' command is the only one we need to check for. The other ones
    // are for testing purposes only, which is why only 'source_file' is checked
    initialize_project(false, false, settings)?;

    let source_file = format!("{SOURCE_DIRECTORY}/{TEST_PACKAGE_NAME}.c");

    assert!(Path::new(BUILD_CONFIG_FILE).is_file());
    assert!(Path::new(SOURCE_DIRECTORY).is_dir());
    assert!(Path::new(INCLUDE_PATH).is_dir());
    assert!(Path::new(&source_file).is_file());
    
    Ok(())
}

/// This will treat the project as if it needs to be rebuilt entirely.
fn test_first_time_compilation(settings : &Settings) -> Result<(), Box<dyn std::error::Error>>
{
    initialize_project(true, false, settings)?;
    let mut tools = Tools::new();
    get_src_files(&mut tools);

    assert_eq!(tools.source_files.len(), TOTAL_SOURCE_FILES);

    let compilation_success = compile_to_object_files(&tools.source_files, 
                                                            &tools.build_config, 
                                                            0, 
                                                            tools.source_files.len());
    assert!(compilation_success);
    assert_eq!(fs::read_dir(BUILD_TABLE_OBJECT_FILE_DIRECTORY)?.count(), TOTAL_SOURCE_FILES);

    let link_success = link_files(&tools.build_config);
    assert!(link_success);

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
        let source_file_to_modify = get_source_file("main.c");
        modify_file_time(source_file_to_modify.as_str())?;
        let mut tools = Tools::new();
        get_src_files(&mut tools);

        // Should be only 1 file that was added, since we modified one file only
        assert_eq!(tools.source_files.len(), 1); 
        let compilation_success = compile_to_object_files(&tools.source_files, 
                                                        &tools.build_config, 
                                                0,
                                                0);

        assert!(compilation_success);
        assert_eq!(fs::read_dir(BUILD_TABLE_OBJECT_FILE_DIRECTORY)?.count(), TOTAL_SOURCE_FILES);

        let link_success = link_files(&tools.build_config);
        assert!(link_success);
    }

    // Modify the header file, once modified, then all of the source files that
    // depended on it need to be recompiled
    {
        let header_file_to_modify = format!("{}/{}", INCLUDE_PATH, "hi.h");
        modify_file_time(header_file_to_modify.as_str())?;
        let mut tools = Tools::new();
        get_src_files(&mut tools);

        // 2 source files depend on the header
        assert_eq!(tools.source_files.len(), 2);

        let compilation_success = compile_to_object_files(&tools.source_files, 
                                                     &tools.build_config, 
                                             0,
                                             0);
        assert!(compilation_success);
        assert_eq!(fs::read_dir(BUILD_TABLE_OBJECT_FILE_DIRECTORY)?.count(), TOTAL_SOURCE_FILES);

        let link_success = link_files(&tools.build_config);
        assert!(link_success);
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
        let compilation_success = compile_to_object_files(&tools.source_files, 
                                                     &tools.build_config, 
                                             0,
                                             0);

        // Compilation should have failed since the invalid file has a error in it
        assert!(!compilation_success); 

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

    let build_config_file_new = format!("{TEST_FILES_DIR}/{BUILD_CONFIG_FILE}");
    fs::copy(build_config_file_new, BUILD_CONFIG_FILE)?;

    let mut tools = Tools::new();
    get_src_files(&mut tools);

    // All of the source files should be recompiled since the build config file has been changed
    assert_eq!(tools.source_files.len(), TOTAL_SOURCE_FILES);
    let compilation_success = compile_to_object_files(&tools.source_files, 
                                                    &tools.build_config, 
                                            0,
                                            0);

    assert!(compilation_success);
    assert_eq!(fs::read_dir(BUILD_TABLE_OBJECT_FILE_DIRECTORY)?.count(), TOTAL_SOURCE_FILES);

    let link_success = link_files(&tools.build_config);
    assert!(link_success);

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

        let compilation_success = compile_to_object_files(&tools.source_files, 
                                                        &tools.build_config, 
                                                0,
                                                0);

        assert!(compilation_success);
        assert_eq!(fs::read_dir(BUILD_TABLE_OBJECT_FILE_DIRECTORY)?.count(), NUM_FILES_AFTER_DELETION);
        assert!(!tools.build_table.contains(format!("{SOURCE_DIRECTORY}/{FILE_TO_BE_DELETED}").as_str()));

        let link_success = link_files(&tools.build_config);
        assert!(link_success);

    }

    Ok(())
}

/// Tests if the project will relink (and not recompile) if the binary has been deleted.
fn test_recompilation_after_deleting_binary(settings : &Settings) -> Result<(), Box<dyn std::error::Error>>
{
    test_first_time_compilation(settings)?;
    fs::remove_file(TEST_PACKAGE_NAME)?;

    let mut tools = Tools::new();
    get_src_files(&mut tools);

    assert_eq!(tools.source_files.len(), 0);
    assert_eq!(fs::read_dir(BUILD_TABLE_OBJECT_FILE_DIRECTORY)?.count(), TOTAL_SOURCE_FILES);

    let link_success = link_files(&tools.build_config);
    assert!(link_success);

    Ok(())
}

/// Tests if the project will recompile correctly after a dependency has been moved/deleted.
fn test_compilation_after_dependency_deletion(settings : &Settings) -> Result<(), Box<dyn std::error::Error>>
{
    test_first_time_compilation(settings).unwrap();

    fs::remove_file(get_dependency_file("hi.h"))?;

    // technically this is invalid since this wouldnt actually compile given
    // a real scenario, but we need to forge the modification date of the 
    // copied files that are not dependent on hi.h to be the same as the original files. 
    // Otherwise quikc will not check if any dependencies changed, since it checks if the
    // source file has changed first. If it does this, then the entire purpose of the test
    // is pointless.
    let mainc_source = get_source_file("main.c");
    let hic_source = get_source_file("hi.c");
    
    let time_modified_mainc = fs::metadata(&mainc_source)?.modified()?;
    let time_modified_hic = fs::metadata(&hic_source)?.modified()?;

    let mainc_no_deps = format!("{TEST_FILES_DIR}/nondeps/main.c");
    let hic_no_deps = format!("{TEST_FILES_DIR}/nondeps/hi.c");


    fs::copy(mainc_no_deps, &mainc_source)?;
    fs::copy(hic_no_deps, &hic_source)?;

    set_file_mtime(&mainc_source, time_modified_mainc.into())?;
    set_file_mtime(&hic_source, time_modified_hic.into())?;

    let mut tools = Tools::new();
    get_src_files(&mut tools);

    // 2 source files had the dependency, with the dependency removed, they were changed, so
    // they need to be recompiled
    assert_eq!(tools.source_files.len(), 2);
    assert_eq!(fs::read_dir(BUILD_TABLE_OBJECT_FILE_DIRECTORY)?.count(), TOTAL_SOURCE_FILES);

    let link_success = link_files(&tools.build_config);
    assert!(link_success);

    Ok(())
}

/// Tests if the Build::new() will correctly initialize the build configuration
/// from the 'Build.toml' file
#[cfg(test)]
fn test_config(settings : &Settings) -> Result<(), Box<dyn std::error::Error>>
{

    test_first_time_compilation(settings)?;

    // Test if default configurations are applied correctly
    let build = Build::new();
    assert!(build.package.debug_build);
    assert_eq!(build.compiler.compiler, match settings.use_clang {
        true => "clang",
        false => "gcc"
    });
    assert_eq!(build.package.name, TEST_PACKAGE_NAME);
    assert_eq!(build.compiler.cstd.unwrap(), DEFAULT_C_STANDARD);
    assert!(build.misc.optimization_level.is_none());
    assert!(build.misc.static_analysis_enabled.is_none());
    assert!(build.compiler.args.is_none());
    assert!(build.linker.args.is_none());
    assert!(build.linker.libraries.is_none());
    assert_eq!(build.compiler.cppstd.unwrap(), DEFAULT_CPP_STANDARD);
    assert_eq!(build.assembler.assembler, build.compiler.compiler);
    assert_eq!(build.assembler.args, build.compiler.args);
    assert!(build.compiler.append_args.is_none());
    assert!(build.linker.append_args.is_none());
    #[cfg(feature = "quikc-nightly")]
    {
        assert!(build.misc.toggle_iwyu.is_none());
    }

    let mut build = Build::new();
    build.package.name = "test".to_string();
    build.package.debug_build = false;
    build.compiler.compiler = "clang++".to_string();
    build.compiler.cstd = Some("c11".to_string());
    build.compiler.cppstd = Some("c++98".to_string());
    build.misc.optimization_level = Some(3);
    build.misc.static_analysis_enabled = Some(true);
    build.compiler.args = Some(vec!["-Wall".to_string(), "-Wextra".to_string()]);
    build.linker.args = Some(vec!["-s".to_string(), "-flto".to_string()]);
    build.linker.libraries = Some(vec![]);
    build.assembler.assembler = "nasm".to_string();
    build.assembler.args = Some(vec!["-felf64".to_string()]);
    build.compiler.append_args = Some(false);
    build.linker.append_args = Some(false);

    #[cfg(feature = "quikc-nightly")]
    {
        build.misc.toggle_iwyu = Some(true);
    }

    write_to_config(&build)?;

    let build = Build::new();
    assert_eq!(build.package.name, "test");
    assert!(!build.package.debug_build);
    assert_eq!(build.compiler.compiler, "clang++");
    assert_eq!(build.compiler.cstd.unwrap(), "-std=c11");
    assert_eq!(build.compiler.cppstd.unwrap(), "-std=c++98");
    assert_eq!(build.misc.optimization_level.unwrap(), 3);
    assert!(build.misc.static_analysis_enabled.unwrap());
    assert_eq!(build.compiler.args.unwrap().len(), 2);
    assert_eq!(build.linker.args.unwrap().len(), 2);
    assert_eq!(build.linker.libraries.unwrap().len(), 0);
    assert_eq!(build.assembler.assembler, "nasm");
    assert_eq!(build.assembler.args, Some(vec!["-felf64".to_string()]));
    assert!(!build.compiler.append_args.unwrap());
    assert!(!build.linker.append_args.unwrap());

    #[cfg(feature = "quikc-nightly")]
    {
        assert!(build.misc.toggle_iwyu.unwrap());
    }

    Ok(())
}

/// Tests if 'Build::linker_with_build_info()' works correctly
fn test_execute_linker_with_build_info(settings : &Settings) -> Result<(), Box<dyn std::error::Error>>
{
    test_first_time_compilation(settings)?;
    let build = Build::new();
    let cmd = build.execute_linker_with_build_info();
    let args = cmd.get_args();

    // default build configuration should have no arguments or libraries passed to it
    assert_eq!(args.len(), 0);

    let linker_args = vec!["-s".to_string(), "-flto".to_string()];
    let library_args = vec!["-lstdc++".to_string()];

    let mut build = Build::new();
    build.linker.args = Some(linker_args.clone());
    build.linker.libraries = Some(library_args.clone());
    let cmd = build.execute_linker_with_build_info();
    let mut args = cmd.get_args();
    assert_eq!(args.len(), linker_args.len() + library_args.len());

    for arg in &linker_args {
        assert!(args.any(|s| s.to_str().unwrap() == arg));
    }
    for arg in &library_args {
        assert!(args.any(|s| s.to_str().unwrap() == arg));
    }

    build.linker.append_args = Some(false);
    let cmd = build.execute_linker_with_build_info();
    let args = cmd.get_args();
    // With append args set to false, the linker arguments should still be the same
    assert_eq!(args.len(), linker_args.len() + library_args.len());

    build.linker.append_args = Some(true);
    let cmd = build.execute_linker_with_build_info();
    let args = cmd.get_args();

    // With append args set to true, the linker arguments should be the same as before
    assert_eq!(args.len(), linker_args.len() + library_args.len());

    build.package.debug_build = false;
    let cmd = build.execute_linker_with_build_info();
    let args = cmd.get_args();

    // debug build set to false so should apply the optimization options
    assert_eq!(args.len(), linker_args.len() + library_args.len() + GCC_AND_CLANG_LINKER_OPTIONS.len());

    build.linker.args = Some(vec![]);
    let cmd = build.execute_linker_with_build_info();
    let args = cmd.get_args();

    assert_eq!(args.len(), library_args.len() + GCC_AND_CLANG_LINKER_OPTIONS.len());

    Ok(())
}

fn run(args : &Vec<&str>) -> String
{
    #[cfg(not(feature = "quikc-nightly"))]
    {
        let cmd = Command::new("cargo")
                        .args(["run", ".."])
                        .args(args)
                        .output()
                        .unwrap()
                        .stdout;
        String::from_utf8_lossy(&cmd).to_string()
    }
    #[cfg(feature = "quikc-nightly")]
    {
        let cmd = Command::new("cargo")
                        .args(["run", "--features", "quikc-nightly", ".."])
                        .args(args)
                        .output()
                        .unwrap()
                        .stdout;
        String::from_utf8_lossy(&cmd).to_string()
    }
}

fn test_cmdline_flags(settings : &Settings) -> Result<(), Box<dyn std::error::Error>>
{
    test_first_time_compilation(settings)?;

    // Test if the version flag works
    #[cfg(not(feature = "quikc-nightly"))]
        assert_eq!(run(&["-v"].to_vec()), concatcp!("quikc v", VERSIONS[0], "\n"));
    #[cfg(feature = "quikc-nightly")]
        assert_eq!(run(&["-v"].to_vec()), concatcp!("quikc-nightly v", NIGHTLY_VERSION, "\n"));

    // Test if -h works (disables verbose output)
    let mut build = Build::new();

    // a compiler unknown to quikc should give a warning/note that the compiler is unknown
    // so default configuration settings cannot be used
    build.compiler.compiler = "not_a_real_compiler".to_string();

    write_to_config(&build)?;
    assert!(!run(&["-h"].to_vec()).contains("cannot use default configuration because compiler vendor is unknown, please supply your own flags."));

    // '-hh' disables all output, except for compiler messages
    assert_eq!(run(&["-hh"].to_vec()), "");

    reset()?;
    test_first_time_compilation(settings)?;

    // '-c' does not link the object files
    assert!(run(&["-c"].to_vec()).contains("Successfully compiled source files to object files"));

    // '-S' with no other arguments assembles all of the source files
    reset()?;
    initialize_project(true, false, settings)?;
    run(&["-S"].to_vec());

    assert_eq!(fs::read_dir(BUILD_TABLE_ASM_DIRECTORY)?.count(), TOTAL_SOURCE_FILES);
    
    reset()?;
    initialize_project(true, false, settings)?;
    run(&["-S", "./src/main.c"].to_vec());

    assert_eq!(fs::read_dir(BUILD_TABLE_ASM_DIRECTORY)?.count(), 1);

    Ok(())
}

/// Tests if 'Build::compiler_with_build_info()' works correctly
fn test_execute_compiler_with_build_info(settings : &Settings) -> Result<(), Box<dyn std::error::Error>>
{
    test_first_time_compilation(settings)?;
    let build = Build::new();
    // Check if default configuration for C on debug builds is correct
    let command = build.execute_compiler_with_build_info("test.c");
    let args = command.get_args().into_iter().map(|s| s.to_str().unwrap()).collect::<Vec<&str>>();

    if settings.use_clang {
        let expected = vec!["-g", DEFAULT_C_STANDARD].into_iter()
                                                    .chain(CLANG_COMPILER_NONEXCLUSIVE_WARNINGS).collect::<Vec<&str>>().into_iter()
                                                    .chain(GCC_AND_CLANG_DIALECT_OPTIONS).collect::<Vec<&str>>();

        for arg in &expected {
            assert!(args.contains(arg), "At line {}: Expected argument '{}' not found in args: {:?}", line!(), arg, args);
            assert!(args.contains(arg));
        }
        
    }
    else {
        let expected = vec!["-g", DEFAULT_C_STANDARD].into_iter()
                                                    .chain(GCC_COMPILER_NONEXCLUSIVE_WARNINGS).collect::<Vec<&str>>().into_iter()
                                                    .chain(GCC_AND_CLANG_DIALECT_OPTIONS).collect::<Vec<&str>>().into_iter()
                                                    .chain(GCC_COMPILER_C_EXCLUSIVE_WARNINGS).collect::<Vec<&str>>();

        for arg in &expected {
            assert!(args.contains(arg), "At line {}: Expected argument '{}' not found in args: {:?}", line!(), arg, args);
            assert!(args.contains(arg));
        }
    }

    // Check if the default configuration for C++ on debug builds is correct
    let command = build.execute_compiler_with_build_info("test.cpp");
    let args = command.get_args().into_iter().map(|s| s.to_str().unwrap()).collect::<Vec<&str>>();
    if settings.use_clang {
        let expected = vec!["-g", DEFAULT_CPP_STANDARD].into_iter()
                                                    .chain(CLANG_COMPILER_NONEXCLUSIVE_WARNINGS).collect::<Vec<&str>>().into_iter()
                                                    .chain(GCC_AND_CLANG_DIALECT_OPTIONS).collect::<Vec<&str>>().into_iter()
                                                    .chain(CLANG_COMPILER_CPP_WARNINGS).collect::<Vec<&str>>().into_iter()
                                                    .chain(GCC_AND_CLANG_CPP_DIALECT_OPTIONS).collect::<Vec<&str>>();

        for arg in &expected {
            assert!(args.contains(arg), "At line {}: Expected argument '{}' not found in args: {:?}", line!(), arg, args);
        }
    }
    else {
        let expected = vec!["-g", DEFAULT_CPP_STANDARD].into_iter()
                                                    .chain(GCC_COMPILER_NONEXCLUSIVE_WARNINGS).collect::<Vec<&str>>().into_iter()
                                                    .chain(GCC_AND_CLANG_DIALECT_OPTIONS).into_iter()
                                                    .chain(GCC_COMPILER_CPP_EXCLUSIVE_WARNINGS).collect::<Vec<&str>>().into_iter()
                                                    .chain(GCC_AND_CLANG_CPP_DIALECT_OPTIONS).collect::<Vec<&str>>();

        for arg in &expected {
            assert!(args.contains(arg), "At line {}: Expected argument '{}' not found in args: {:?}", line!(), arg, args);
        }
    }

    let mut build = Build::new();
    build.package.debug_build = false;

    // Check if the default configuration for C on release builds is correct
    let command = build.execute_compiler_with_build_info("test.c");
    let args = command.get_args().into_iter().map(|s| s.to_str().unwrap()).collect::<Vec<&str>>();

    if settings.use_clang {
        let expected = vec![DEFAULT_C_STANDARD].into_iter()
                                                    .chain(CLANG_COMPILER_NONEXCLUSIVE_WARNINGS).collect::<Vec<&str>>().into_iter()
                                                    .chain(GCC_AND_CLANG_OPTIMIZATION_OPTIONS).collect::<Vec<&str>>().into_iter()
                                                    .chain(GCC_AND_CLANG_DIALECT_OPTIONS).collect::<Vec<&str>>();

        for arg in &expected {
            assert!(args.contains(arg), "At line {}: Expected argument '{}' not found in args: {:?}", line!(), arg, args);
        }
        
    }
    else {
        let expected = vec![DEFAULT_C_STANDARD].into_iter()
                                                    .chain(GCC_COMPILER_NONEXCLUSIVE_WARNINGS).collect::<Vec<&str>>().into_iter()
                                                    .chain(GCC_AND_CLANG_DIALECT_OPTIONS).collect::<Vec<&str>>().into_iter()
                                                    .chain(GCC_AND_CLANG_OPTIMIZATION_OPTIONS).collect::<Vec<&str>>().into_iter()
                                                    .chain(GCC_COMPILER_C_EXCLUSIVE_WARNINGS).collect::<Vec<&str>>();

        for arg in &expected {
            assert!(args.contains(arg), "At line {}: Expected argument '{}' not found in args: {:?}", line!(), arg, args);
        }
    }


    // Check if the default configuration for C++ on release builds is correct
    let command = build.execute_compiler_with_build_info("test.cpp");
    let args = command.get_args().into_iter().map(|s| s.to_str().unwrap()).collect::<Vec<&str>>();
    if settings.use_clang {
        let expected = vec![DEFAULT_CPP_STANDARD].into_iter()
                                                    .chain(CLANG_COMPILER_NONEXCLUSIVE_WARNINGS).collect::<Vec<&str>>().into_iter()
                                                    .chain(GCC_AND_CLANG_DIALECT_OPTIONS).collect::<Vec<&str>>().into_iter()
                                                    .chain(CLANG_COMPILER_CPP_WARNINGS).collect::<Vec<&str>>().into_iter()
                                                    .chain(GCC_AND_CLANG_OPTIMIZATION_OPTIONS).collect::<Vec<&str>>().into_iter()
                                                    .chain(GCC_AND_CLANG_CPP_DIALECT_OPTIONS).collect::<Vec<&str>>();

        for arg in &expected {
            assert!(args.contains(arg), "At line {}: Expected argument '{}' not found in args: {:?}", line!(), arg, args);
        }
    }
    else {
        let expected = vec![DEFAULT_CPP_STANDARD].into_iter()
                                                    .chain(GCC_COMPILER_NONEXCLUSIVE_WARNINGS).collect::<Vec<&str>>().into_iter()
                                                    .chain(GCC_AND_CLANG_DIALECT_OPTIONS).into_iter()
                                                    .chain(GCC_COMPILER_CPP_EXCLUSIVE_WARNINGS).collect::<Vec<&str>>().into_iter()
                                                    .chain(GCC_AND_CLANG_OPTIMIZATION_OPTIONS).collect::<Vec<&str>>().into_iter()
                                                    .chain(GCC_AND_CLANG_CPP_DIALECT_OPTIONS).collect::<Vec<&str>>();

        for arg in &expected {
            assert!(args.contains(arg), "At line {}: Expected argument '{}' not found in args: {:?}", line!(), arg, args);
        }
    }

    let mut build = Build::new();
    let chosen_cpp_standard = "-std=c++14";
    let chosen_c_standard = "-std=c99";

    build.package.debug_build = false;
    build.compiler.cppstd = Some(chosen_cpp_standard.to_string());
    build.compiler.cstd = Some(chosen_c_standard.to_string());

    build.misc.optimization_level = Some(3);
    build.misc.static_analysis_enabled = Some(true);

    // Check if custom configuration rules override the default ones (they should)
    let command = build.execute_compiler_with_build_info("test.c");
    let args = command.get_args().into_iter().map(|s| s.to_str().unwrap()).collect::<Vec<&str>>();

    // Check if the custom configuration rules work for C
    if settings.use_clang {
        let expected = vec![chosen_c_standard].into_iter()
                                                    .chain(CLANG_COMPILER_NONEXCLUSIVE_WARNINGS).collect::<Vec<&str>>().into_iter()
                                                    .chain(GCC_AND_CLANG_ENHANCED_OPTIMIZATION_OPTIONS).collect::<Vec<&str>>().into_iter()
                                                    .chain(GCC_AND_CLANG_DIALECT_OPTIONS).collect::<Vec<&str>>();

        for arg in &expected {
            assert!(args.contains(arg), "At line {}: Expected argument '{}' not found in args: {:?}", line!(), arg, args);
        }
        
    }
    else {
        let expected = vec![chosen_c_standard].into_iter()
                                                    .chain(GCC_COMPILER_NONEXCLUSIVE_WARNINGS).collect::<Vec<&str>>().into_iter()
                                                    .chain(GCC_AND_CLANG_DIALECT_OPTIONS).collect::<Vec<&str>>().into_iter()
                                                    .chain(GCC_AND_CLANG_ENHANCED_OPTIMIZATION_OPTIONS).collect::<Vec<&str>>().into_iter()
                                                    .chain(GCC_COMPILER_C_EXCLUSIVE_WARNINGS).collect::<Vec<&str>>();

        for arg in &expected {
            assert!(args.contains(arg), "At line {}: Expected argument '{}' not found in args: {:?}", line!(), arg, args);
        }

    }

    let command = build.execute_compiler_with_build_info("test.cpp");
    let args = command.get_args().into_iter().map(|s| s.to_str().unwrap()).collect::<Vec<&str>>();

    // Check if the custom configuration rules work for C++
    if settings.use_clang {
        let expected = vec![chosen_cpp_standard].into_iter()
                                                    .chain(CLANG_COMPILER_NONEXCLUSIVE_WARNINGS).collect::<Vec<&str>>().into_iter()
                                                    .chain(GCC_AND_CLANG_DIALECT_OPTIONS).collect::<Vec<&str>>().into_iter()
                                                    .chain(CLANG_COMPILER_CPP_WARNINGS).collect::<Vec<&str>>().into_iter()
                                                    .chain(GCC_AND_CLANG_ENHANCED_OPTIMIZATION_OPTIONS).collect::<Vec<&str>>().into_iter()
                                                    .chain(GCC_AND_CLANG_CPP_DIALECT_OPTIONS).collect::<Vec<&str>>();

        for arg in &expected {
            assert!(args.contains(arg), "At line {}: Expected argument '{}' not found in args: {:?}", line!(), arg, args);
        }
    }
    else {
        let expected = vec![chosen_cpp_standard].into_iter()
                                                    .chain(GCC_COMPILER_NONEXCLUSIVE_WARNINGS).collect::<Vec<&str>>().into_iter()
                                                    .chain(GCC_AND_CLANG_DIALECT_OPTIONS).into_iter()
                                                    .chain(GCC_COMPILER_CPP_EXCLUSIVE_WARNINGS).collect::<Vec<&str>>().into_iter()
                                                    .chain(GCC_AND_CLANG_ENHANCED_OPTIMIZATION_OPTIONS).collect::<Vec<&str>>().into_iter()
                                                    .chain(GCC_AND_CLANG_CPP_DIALECT_OPTIONS).collect::<Vec<&str>>();

        for arg in &expected {
            assert!(args.contains(arg), "At line {}: Expected argument '{}' not found in args: {:?}", line!(), arg, args);
        }
    }

    // Check if default arguments are removed when the user defines their own arguments
    // (note: this won't remove all of them, e.g., standard, debug info, etc).

    let mut build = Build::new();
    build.compiler.args = Some(vec!["-Wall".to_string(), "-Wextra".to_string()]);

    let command = build.execute_compiler_with_build_info("test.c");
    let args = command.get_args().into_iter().map(|s| s.to_str().unwrap()).collect::<Vec<&str>>();

    assert!(args.contains(&DEFAULT_C_STANDARD), "At line {}: Expected argument '{}' not found in args: {:?}", line!(), DEFAULT_C_STANDARD, args);
    assert!(args.contains(&"-g"), "At line {}: Expected argument '-g' not found in args: {:?}", line!(), args);
    for arg in &build.compiler.args.unwrap() {
        assert!(args.contains(&arg.as_str()), "At line {}: Argument '{}' should be present in args: {:?}", line!(), arg, args);
    }

    let mut build = Build::new();

    // not valid but since we arent invoking the compiler it does not matter. we just want to see if the arguments will be
    // appended when the actual command will be run
    let v = vec!["-Notavalidargument", "-Appendworks!"];
    build.compiler.args = Some(vec!["-Notavalidargument".to_string(), "-Appendworks!".to_string()]);

    build.compiler.append_args = Some(true);

    let command = build.execute_compiler_with_build_info("test.c");
    let args = command.get_args().into_iter().map(|s| s.to_str().unwrap()).collect::<Vec<&str>>();

    if settings.use_clang {
        let expected = v.into_iter()
                                    .chain(CLANG_COMPILER_NONEXCLUSIVE_WARNINGS).collect::<Vec<&str>>().into_iter()
                                    .chain(GCC_AND_CLANG_DIALECT_OPTIONS).collect::<Vec<&str>>().into_iter();

        assert!(args.contains(&DEFAULT_C_STANDARD), "At line {}: Expected argument '{}' not found in args: {:?}", line!(), DEFAULT_C_STANDARD, args);
        assert!(args.contains(&"-g"), "At line {}: Expected argument '-g' not found in args: {:?}", line!(), args);

        for arg in expected {
            assert!(args.contains(&arg), "At line {}: Argument '{}' should be present in args: {:?}", line!(), arg, args);
        }
    }
    else {
        let expected = v.into_iter()
                                   .chain(GCC_COMPILER_NONEXCLUSIVE_WARNINGS).collect::<Vec<&str>>().into_iter()
                                   .chain(GCC_AND_CLANG_DIALECT_OPTIONS).collect::<Vec<&str>>().into_iter()
                                   .chain(GCC_COMPILER_C_EXCLUSIVE_WARNINGS).collect::<Vec<&str>>();

        assert!(args.contains(&DEFAULT_C_STANDARD), "At line {}: Expected argument '{}' not found in args: {:?}", line!(), DEFAULT_C_STANDARD, args);
        assert!(args.contains(&"-g"), "At line {}: Expected argument '-g' not found in args: {:?}", line!(), args);
        for arg in expected {
            assert!(args.contains(&arg), "At line {}: Argument '{}' should be present in args: {:?}", line!(), arg, args);
        }
    }

    Ok(())
}

