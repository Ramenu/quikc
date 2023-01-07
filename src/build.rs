use std::{fs::{self}, process::Command, path::Path};
use color_print::{cprintln, cformat};
use serde_derive::Deserialize;

use crate::{defaultbuild::{GCC_COMPILER_NONEXCLUSIVE_WARNINGS, GCC_COMPILER_C_EXCLUSIVE_WARNINGS, GCC_COMPILER_CPP_DIALECT_OPTIONS, GCC_COMPILER_CPP_EXCLUSIVE_WARNINGS, GCC_STATIC_ANALYSIS_OPTIONS, GCC_AND_CLANG_DIALECT_OPTIONS, CLANG_COMPILER_NONEXCLUSIVE_WARNINGS, CLANG_COMPILER_CPP_WARNINGS, GCC_AND_CLANG_OPTIMIZATION_OPTIONS, GCC_AND_CLANG_ENHANCED_OPTIMIZATION_OPTIONS, GCC_AND_CLANG_LINKER_OPTIONS, GCC_AND_CLANG_CPP_DIALECT_OPTIONS}, compiler::{self, use_default_compiler_configuration}, buildtable::BUILD_TABLE_OBJECT_FILE_DIRECTORY, linker};

const BUILD_CONFIG_FILE : &str = "./Build.toml";
const BUILD_CONFIG_CACHE_FILE : &str = "./buildinfo/.buildcache";


#[derive(Deserialize, PartialEq)]
struct Package
{
    name : String,
    debug_build : bool
}

#[derive(Deserialize, PartialEq)]
struct Compiler
{
    compiler : Option<String>,
    args : Option<Vec<String>>,
    cstd : Option<String>,
    cppstd : Option<String>
}

#[derive(Deserialize, PartialEq)]
struct Linker
{
    args : Option<Vec<String>>,
    libraries : Option<Vec<String>>
}

#[derive(Deserialize, PartialEq)]
struct Misc
{
    optimization_level : Option<u32>,
    static_analysis_enabled : Option<bool>
}

#[derive(Deserialize,PartialEq)]
pub struct Build
{
    package : Package,
    compiler : Compiler,
    linker : Linker,
    misc : Misc
}


impl Build
{
    #[inline]
    pub fn new() -> Build
    {
        let file_contents = fs::read_to_string(BUILD_CONFIG_FILE).expect("Failed to read from build configuration file");
        let mut toml_config = toml::from_str(&file_contents).expect("Failed to parse build configuration file");
        let cached_file_contents = fs::read_to_string(BUILD_CONFIG_CACHE_FILE)
                                              .expect("Failed to read from cached build configuration file");

        if !Path::new(BUILD_CONFIG_CACHE_FILE).is_file() {
            fs::copy(BUILD_CONFIG_FILE, BUILD_CONFIG_CACHE_FILE).expect("Failed to cache build configuration file");
            return toml_config;
        }


        let cached_toml = toml::from_str(&cached_file_contents).expect("Failed to parse cached build configuration file");

        // Check if the configuration file has changed since the last build, if so we need to remove all the object
        // files to recompile them again
        if toml_config != cached_toml {
            if Path::new(BUILD_TABLE_OBJECT_FILE_DIRECTORY).is_dir() {
                fs::remove_dir_all(BUILD_TABLE_OBJECT_FILE_DIRECTORY).expect("Failed to remove build table object file directory");
            }
        }
        fs::copy(BUILD_CONFIG_FILE, BUILD_CONFIG_CACHE_FILE).expect("Failed to cache build configuration file");

        // If a default compiler is not provided, select one automatically
        if toml_config.compiler.compiler.is_none() {
            toml_config.compiler.compiler = compiler::select_default_compiler();
            if toml_config.compiler.compiler.is_none() {
                eprintln!("{}", cformat!("<red><bold>error:</bold></red> Could not find a default compiler to use.
                                          Please specify your own in the 'build.toml' file"));
                std::process::exit(1);
            }
            else {
                cprintln!("<bold><yellow>note:</yellow></bold> Compiler not set in 'Build.toml', using {} as default", 
                          toml_config.compiler.compiler.as_ref().unwrap());
            }
        }

        return toml_config;
    }

    pub fn execute_compiler_with_build_info(&self, file : &String) -> Command
    {
        let compiler_name = self.compiler.compiler.as_ref().unwrap();
        let compiler_args = &self.compiler.args;

        let mut cmd = Command::new(compiler_name);
        // The only variable that cannot really be overridden is whether or not
        // the build is being compiled in debug mode
        if self.package.debug_build {
            cmd.arg("-g");
        }

        // If the default configuration variable is set to true, use the default arguments
        // (note this doesnt support MSVC)
        if use_default_compiler_configuration(compiler_args) {

            let is_c_source_file = compiler::is_c_source_file(file);

            if !is_c_source_file {
                cmd.args(GCC_AND_CLANG_CPP_DIALECT_OPTIONS);

                // specify c++ standard if given
                if self.compiler.cppstd.is_some() {
                    cmd.arg(format!("-std={}", self.compiler.cppstd.as_ref().unwrap()));
                }
                else {
                    cmd.arg("-std=c++20"); // default to c++20
                }
            }
            else {
                // specify c standard if given
                if self.compiler.cstd.is_some() {
                    cmd.arg(format!("-std={}", self.compiler.cstd.as_ref().unwrap()));
                }
                else {
                    cmd.arg("-std=c17"); // default to c17
                }
            }

            // Default configuration only supported on gcc and clang
            if compiler::is_gcc_or_clang(compiler_name) {

                cmd.args(GCC_AND_CLANG_DIALECT_OPTIONS);

                // Check if the optimization level is not specified, if not
                // then apply the regular optimizations on release builds only
                if self.misc.optimization_level.is_none() {
                    if !self.package.debug_build {
                        cmd.args(GCC_AND_CLANG_OPTIMIZATION_OPTIONS);
                    }
                }
                // otherwise leave it up to the user to specify the optimization level
                else {
                    let opt_level = self.misc.optimization_level.unwrap();
                    if opt_level == 2 {
                        cmd.args(GCC_AND_CLANG_OPTIMIZATION_OPTIONS);
                    }

                    else if opt_level == 3 {
                        cmd.args(GCC_AND_CLANG_ENHANCED_OPTIMIZATION_OPTIONS);
                    }
                }

                
                match compiler_name.as_str() {
                    "gcc"|"g++" => {
                        cmd.args(GCC_COMPILER_NONEXCLUSIVE_WARNINGS);
                        if is_c_source_file {
                            cmd.args(GCC_COMPILER_C_EXCLUSIVE_WARNINGS);
                        }
                        else {
                            cmd.args(GCC_COMPILER_CPP_DIALECT_OPTIONS);
                            cmd.args(GCC_COMPILER_CPP_EXCLUSIVE_WARNINGS);
                        }
                        if self.misc.static_analysis_enabled.is_some() {
                            if self.misc.static_analysis_enabled.unwrap() {
                                cmd.args(GCC_STATIC_ANALYSIS_OPTIONS);
                            }
                        }
                    }
                    "clang"|"clang++" => {
                        cmd.args(CLANG_COMPILER_NONEXCLUSIVE_WARNINGS);
                        if !is_c_source_file {
                            cmd.args(CLANG_COMPILER_CPP_WARNINGS);
                        }
                    },
                    _ => ()
                }   
            }
            else {
                cprintln!("<bold><yellow>note:</yellow></bold> Cannot use default configuration because
                           compiler vendor is unknown, please supply your own flags.");
            }
            return cmd;
        }

        // If default configuration is not set, then use the user's custom flags
        if !compiler_args.as_ref().unwrap().is_empty() {
            cmd.args(compiler_args.as_ref().unwrap().iter());
        }
        return cmd;

    }

    pub fn execute_linker_with_build_info(&self) -> Command
    {
        let compiler_name = self.compiler.compiler.as_ref().unwrap();
        let linker_libraries = self.linker.libraries.as_ref();

        let mut cmd = Command::new(compiler_name);

        if linker::use_default_linker_configuration(&self.linker.args) {
            // apply linker optimizations on release builds only
            if !self.package.debug_build {
                cmd.args(GCC_AND_CLANG_LINKER_OPTIONS);
            }
        }
        else {
            let linker_args = self.linker.args.as_ref().unwrap();
            cmd.args(linker_args.iter());
        }
        if linker_libraries.is_some() {
            if !linker_libraries.unwrap().is_empty() {
                cmd.args(linker_libraries.unwrap().iter());
            }
        }
        return cmd;

    }

    #[inline]
    pub fn get_package_name(&self) -> &String {
        return &self.package.name;
    }

    #[inline]
    pub fn get_compiler_name(&self) -> &String {
        return &self.compiler.compiler.as_ref().unwrap();
    }
}