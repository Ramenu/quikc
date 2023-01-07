use std::{fs, process::Command};
use color_print::cprintln;
use serde_derive::Deserialize;

use crate::{defaultbuild::{GCC_COMPILER_NONEXCLUSIVE_WARNINGS, GCC_COMPILER_C_EXCLUSIVE_WARNINGS, GCC_COMPILER_CPP_DIALECT_OPTIONS, GCC_COMPILER_CPP_EXCLUSIVE_WARNINGS, GCC_STATIC_ANALYSIS_OPTIONS, GCC_AND_CLANG_DIALECT_OPTIONS, CLANG_COMPILER_NONEXCLUSIVE_WARNINGS, CLANG_COMPILER_CPP_WARNINGS, GCC_AND_CLANG_OPTIMIZATION_OPTIONS, GCC_AND_CLANG_ENHANCED_OPTIMIZATION_OPTIONS, GCC_AND_CLANG_LINKER_OPTIONS, GCC_AND_CLANG_CPP_DIALECT_OPTIONS}, compiler};

const BUILD_CONFIG_FILE : &str = "./Build.toml";


#[derive(Deserialize)]
struct Package
{
    name : String,
    debug_build : bool
}

#[derive(Deserialize)]
struct Compiler
{
    compiler : String,
    args : Vec<String>
}

#[derive(Deserialize)]
struct Linker
{
    args : Vec<String>,
    libraries : Vec<String>
}

#[derive(Deserialize)]
struct Misc
{
    optimization_level : u32,
    static_analysis_enabled : bool
}

#[derive(Deserialize)]
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
        return toml::from_str(&file_contents).expect("Failed to parse build configuration file");
    }

    pub fn execute_compiler_with_build_info(&self, file : &String) -> Command
    {
        let mut cmd = Command::new(&self.compiler.compiler);
        // The only variable that cannot really be overridden is whether or not
        // the build is being compiled in debug mode
        if self.package.debug_build {
            cmd.arg("-g");
        }

        let use_default_compiler_config = self.compiler.args.is_empty();

        // If the default configuration variable is set to true, use the default arguments
        // (note this doesnt support MSVC)
        if use_default_compiler_config {

            let is_c_source_file = compiler::is_c_source_file(file);
            if !is_c_source_file {
                cmd.args(GCC_AND_CLANG_CPP_DIALECT_OPTIONS);
            }
            // Default configuration only supported on gcc and clang
            if compiler::is_gcc_or_clang(&self.compiler.compiler) {

                cmd.args(GCC_AND_CLANG_DIALECT_OPTIONS);

                if self.misc.optimization_level == 2 {
                    cmd.args(GCC_AND_CLANG_OPTIMIZATION_OPTIONS);
                }

                else if self.misc.optimization_level == 3 {
                    cmd.args(GCC_AND_CLANG_ENHANCED_OPTIMIZATION_OPTIONS);
                }
                
                match self.compiler.compiler.as_str() {
                    "gcc"|"g++" => {
                        cmd.args(GCC_COMPILER_NONEXCLUSIVE_WARNINGS);
                        if is_c_source_file {
                            cmd.args(GCC_COMPILER_C_EXCLUSIVE_WARNINGS);
                        }
                        else {
                            cmd.args(GCC_COMPILER_CPP_DIALECT_OPTIONS);
                            cmd.args(GCC_COMPILER_CPP_EXCLUSIVE_WARNINGS);
                        }
                        if self.misc.static_analysis_enabled {
                            cmd.args(GCC_STATIC_ANALYSIS_OPTIONS);
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
        if !self.compiler.args.is_empty() {
            cmd.args(self.compiler.args.iter());
        }
        return cmd;

    }

    pub fn execute_linker_with_build_info(&self) -> Command
    {
        let mut cmd = Command::new(&self.compiler.compiler);
        let use_default_linker_configuration = self.linker.args.is_empty();

        if use_default_linker_configuration {
            if !self.package.debug_build {
                cmd.args(GCC_AND_CLANG_LINKER_OPTIONS);
            }
        }
        else {
            cmd.args(self.linker.args.iter());
        }
        if !self.linker.libraries.is_empty() {
            cmd.args(self.linker.libraries.iter());
        }
        return cmd;

    }

    #[inline]
    pub fn get_package_name(&self) -> &String {
        return &self.package.name;
    }

    #[inline]
    pub fn get_compiler_name(&self) -> &String {
        return &self.compiler.compiler;
    }
}