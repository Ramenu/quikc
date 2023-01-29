use std::{fs::{self}, process::Command, path::Path};
use color_print::{cprintln, cformat};
use serde_derive::Deserialize;
#[cfg(feature = "quikc-nightly")] 
    use crate::example;
#[cfg(feature = "quikc-nightly")] 
    use crate::logger;

use crate::{defaultbuild::{GCC_COMPILER_NONEXCLUSIVE_WARNINGS, GCC_COMPILER_C_EXCLUSIVE_WARNINGS, GCC_COMPILER_CPP_DIALECT_OPTIONS, GCC_COMPILER_CPP_EXCLUSIVE_WARNINGS, GCC_STATIC_ANALYSIS_OPTIONS, GCC_AND_CLANG_DIALECT_OPTIONS, CLANG_COMPILER_NONEXCLUSIVE_WARNINGS, CLANG_COMPILER_CPP_WARNINGS, GCC_AND_CLANG_OPTIMIZATION_OPTIONS, GCC_AND_CLANG_ENHANCED_OPTIMIZATION_OPTIONS, GCC_AND_CLANG_LINKER_OPTIONS, GCC_AND_CLANG_CPP_DIALECT_OPTIONS}, compiler::{self, use_default_compiler_configuration, select_default_compiler, INCLUDE_PATH}, buildtable::{BUILD_TABLE_OBJECT_FILE_DIRECTORY, BUILD_TABLE_DIRECTORY, BUILD_TABLE_FILE}, linker, SOURCE_DIRECTORY, QuikcFlags, flags};

pub const BUILD_CONFIG_FILE : &str = "./Build.toml";
pub const BUILD_CONFIG_CACHE_FILE : &str = "./buildinfo/.buildcache";


#[derive(Deserialize, PartialEq, Default)]
struct Package
{
    name : String,
    debug_build : bool
}

#[derive(Deserialize, PartialEq, Default)]
pub struct Compiler
{
    compiler : String,
    args : Option<Vec<String>>,
    cstd : Option<String>,
    cppstd : Option<String>,
    #[cfg(feature = "quikc-nightly")]
    append_args : Option<bool>
}

#[derive(Deserialize, PartialEq, Default)]
pub struct Linker
{
    args : Option<Vec<String>>,
    libraries : Option<Vec<String>>,
    #[cfg(feature = "quikc-nightly")]
    append_args : Option<bool>
}

#[derive(Deserialize, PartialEq, Default, Clone, Copy)]
struct Misc
{
    optimization_level : Option<u32>,
    static_analysis_enabled : Option<bool>,
    #[cfg(feature = "quikc-nightly")]
    toggle_iwyu : Option<bool> // Not recommended to be toggled as it contains a lot of bugs
}

#[derive(Deserialize, PartialEq, Default)]
pub struct BuildOption
{
    package : Package,
    compiler : Option<Compiler>,
    linker : Option<Linker>,
    misc : Option<Misc>
}

#[derive(Deserialize, PartialEq, Default)]
pub struct Build
{
    package : Package,
    compiler : Compiler,
    linker : Linker,
    misc : Misc
}

#[cfg(feature = "quikc-nightly")]
#[inline]
fn warning(message : &str)
{
    cprintln!("<bold><yellow>warning</yellow>:</bold> {}", message);
}

impl Build
{
    #[inline]
    pub fn new() -> Build
    {
        // Include path and source directory are required as that is where the compiler will look for files
        if !Path::new(INCLUDE_PATH).exists() {
            eprintln!("{}", cformat!("<bold><red>error</red>:</bold> './include' directory not found\nTerminating program."));
            std::process::exit(1);
        }

        if !Path::new(SOURCE_DIRECTORY).exists() {
            eprintln!("{}", cformat!("<bold><red>error</red>:</bold> './src' directory not found\nTerminating program."));
            std::process::exit(1);
        }

        if !Path::new(BUILD_TABLE_DIRECTORY).exists() {
            fs::create_dir(BUILD_TABLE_DIRECTORY).expect("Failed to create directory")
        }

        if !Path::new(BUILD_CONFIG_FILE).exists() {
            eprintln!("{}", cformat!("<bold><red>error</red>:</bold> 'Build.toml' not found in working directory\nTerminating program."));
            std::process::exit(1);
        }
        let file_contents = fs::read_to_string(BUILD_CONFIG_FILE).expect("Failed to read from build configuration file");
        let toml_config : BuildOption = toml::from_str(&file_contents).expect("Failed to parse build configuration file");
        let mut cached_toml : Option<BuildOption> = None;

        if Path::new(BUILD_CONFIG_CACHE_FILE).exists() {
            let cached_file_contents = fs::read_to_string(BUILD_CONFIG_CACHE_FILE)
                                                .expect("Failed to read from cached build configuration file");
            cached_toml = Some(toml::from_str(&cached_file_contents).expect("Failed to parse cached build configuration file"));
        }
        else {
            fs::copy(BUILD_CONFIG_FILE, BUILD_CONFIG_CACHE_FILE).expect("Failed to cache build configuration file");
        }

        let mut config = Build::default();

        // If a default compiler is not provided, select one automatically
        if toml_config.compiler.is_some() {
            if toml_config.compiler.as_ref().unwrap().compiler.is_empty() {
                config.compiler.compiler = select_default_compiler().to_string();
                if flags()&QuikcFlags::HIDE_VERBOSE_OUTPUT == QuikcFlags::NONE {
                    cprintln!("<bold><yellow>note</yellow>:</bold> compiler not specified in 'Build.toml', using {} as default",
                            config.compiler.compiler);
                }
            }
            else {
                config.compiler.compiler = toml_config.compiler.as_ref().unwrap().compiler.to_owned();
            }
            let config_ref = toml_config.compiler.as_ref().unwrap();
            config.compiler.args = toml_config.compiler.as_ref().unwrap().args.to_owned();

            config.compiler.cppstd = Some(if config_ref.cppstd.is_none() {"-std=c++20".to_string()} 
                                          else {format!("-std={}", config_ref.cppstd.as_ref().unwrap())});
            config.compiler.cstd = Some(if config_ref.cstd.is_none() {"-std=c17".to_string()} 
                                        else {format!("-std={}", config_ref.cstd.as_ref().unwrap())});
            #[cfg(feature = "quikc-nightly")] {
                config.compiler.append_args = toml_config.compiler.as_ref().unwrap().append_args;
            }
        }
        else {
            config.compiler.compiler = select_default_compiler().to_string();
            if flags()&QuikcFlags::HIDE_VERBOSE_OUTPUT == QuikcFlags::NONE {
                cprintln!("<bold><yellow>note</yellow>:</bold> compiler not specified in 'Build.toml', using {} as default",
                        config.compiler.compiler);
            }
        }

        // Make sure the cached toml exists before comparing the files
        if let Some(cached_toml) = cached_toml {
            // Check if the configuration file has changed since the last build, if so we need to remove all the object
            // files to recompile them again (as well as the 'table.toml' file)
            if toml_config != cached_toml && Path::new(BUILD_TABLE_OBJECT_FILE_DIRECTORY).is_dir() {
                fs::remove_dir_all(BUILD_TABLE_OBJECT_FILE_DIRECTORY).expect("Failed to remove build table object file directory");
                fs::remove_file(BUILD_TABLE_FILE).expect("Failed to remove build table file");
                fs::copy(BUILD_CONFIG_FILE, BUILD_CONFIG_CACHE_FILE).expect("Failed to copy from build table file");
            }
        }

        config.misc = match &toml_config.misc {
            Some(misc) => *misc,
            None => Misc {
                optimization_level : None,
                static_analysis_enabled : None,
                #[cfg(feature = "quikc-nightly")]
                toggle_iwyu : None
            }
        };

        config.linker = match toml_config.linker {
            Some(linker) => linker,
            None => Linker {
                args : None,
                libraries : None,
                #[cfg(feature = "quikc-nightly")]
                append_args : None
            }
        };

        config.package.name = toml_config.package.name;
        config.package.debug_build = toml_config.package.debug_build;
        
        // If using nightly features, notify the user that some of the features can break compilation
        #[cfg(feature = "quikc-nightly")]
        {
            if flags()&QuikcFlags::HIDE_VERBOSE_OUTPUT == QuikcFlags::NONE {
                if let Some(true) = config.misc.toggle_iwyu {
                    warning("'include what you use' WILL refactor your code to only include the headers that are needed.\n\
                            This may cause your code to not compile. For more information see: https://github.com/include-what-you-use/include-what-you-use");
                }
                if let Some(true) = config.compiler.append_args {
                    let args_specified = config.compiler.args().is_some();

                    // give a warning here as this can override the default configuration (which the user probably
                    // did not mean to do)
                    if !args_specified && (flags()&QuikcFlags::HIDE_VERBOSE_OUTPUT == QuikcFlags::NONE) {
                        logger::warning("append_args is set to true, but no args field for the compiler could be found. Try adding this line:");
                        example::print_missing_field("args = []", example::FieldType::CompilerArgs);
                    }
                }
                if let Some(true) = config.linker.append_args {
                    let args_specified = config.linker.args.is_some();

                    // give a warning here as this can override the default configuration (which the user probably
                    // did not mean to do)
                    if !args_specified && (flags()&QuikcFlags::HIDE_VERBOSE_OUTPUT == QuikcFlags::NONE) {
                        logger::warning("append_args is set to true, but no args field for the linker could be found. Try adding this line:");
                        example::print_missing_field("args = []", example::FieldType::LinkerArgs);
                    }
                }
            }
        }
        config
    }

    pub fn execute_compiler_with_build_info(&self, file : &str) -> Command
    {
        let compiler_args = &self.compiler.args;

        let mut cmd = Command::new(&self.compiler.compiler);
        // The only variable that cannot really be overridden is whether or not
        // the build is being compiled in debug mode
        if self.package.debug_build {
            cmd.arg("-g");
        }

        let is_c_source_file = compiler::is_c_source_file(file);

        if !is_c_source_file {
            cmd.arg(self.compiler.cppstd.as_ref().unwrap());
        }
        else {
            cmd.arg(self.compiler.cstd.as_ref().unwrap());
        }

        // If the default configuration variable is set to true, use the default arguments
        // (note this doesnt support MSVC)
        if use_default_compiler_configuration(&self.compiler) {
            if !is_c_source_file {
                cmd.args(GCC_AND_CLANG_CPP_DIALECT_OPTIONS);
            }

            // Default configuration only supported on gcc and clang
            if compiler::is_gcc_or_clang(&self.compiler.compiler) {

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
                        if self.misc.static_analysis_enabled.is_some() && self.misc.static_analysis_enabled.unwrap() {
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
                cprintln!("<bold><yellow>note:</yellow></bold> cannot use default configuration because
                           compiler vendor is unknown, please supply your own flags.");
            }

            // append arguments if the flag is set
            #[cfg(feature = "quikc-nightly")]
            {
                if let Some(true) = self.misc.toggle_iwyu {
                    cmd.args(compiler_args.as_ref().unwrap().iter());
                }
            }
            return cmd;
        }

        // If default configuration is not set, then use the user's custom flags
        if let Some(compiler_args) = compiler_args {
            cmd.args(compiler_args.iter());
        }
        cmd

    }

    pub fn execute_linker_with_build_info(&self) -> Command
    {
        let linker_libraries = self.linker.libraries.as_ref();

        let mut cmd = Command::new(&self.compiler.compiler);

        if linker::use_default_linker_configuration(&self.linker) {
            // apply linker optimizations on release builds only
            if !self.package.debug_build {
                cmd.args(GCC_AND_CLANG_LINKER_OPTIONS);
            }
            // append arguments if the flag is set
            #[cfg(feature = "quikc-nightly")]
            {
                if let Some(true) = self.linker.append_args {
                    cmd.args(self.linker.args.as_ref().unwrap().iter());
                }
            }
        }
        else {
            let linker_args = self.linker.args.as_ref().unwrap();
            cmd.args(linker_args.iter());
        }
        if let Some(linker_libraries) = linker_libraries {
            cmd.args(linker_libraries.iter());
        }
        cmd

    }

    #[inline]
    pub fn get_standard(&self, file_name : &str) -> &String {
        if compiler::is_c_source_file(file_name) {
            self.compiler.cstd.as_ref().unwrap()
        }
        else {
            self.compiler.cppstd.as_ref().unwrap()
        }
    }

    #[inline]
    pub fn get_package_name(&self) -> &String {
        &self.package.name
    }

    #[inline]
    pub fn is_debug_build(&self) -> bool {
        self.package.debug_build
    }

    #[inline]
    pub fn get_compiler_name(&self) -> &String {
        &self.compiler.compiler
    }

    #[cfg(feature = "quikc-nightly")]
    #[inline]
    pub fn iwyu_enabled(&self) -> bool {
        self.misc.toggle_iwyu.unwrap_or(false)
    }

}


impl Compiler
{
    pub fn args(&self) -> Option<&Vec<String>> {
        self.args.as_ref()
    }

    pub fn cstd(&self) -> &String {
        self.cstd.as_ref().unwrap()
    }

    pub fn cppstd(&self) -> &String {
        self.cppstd.as_ref().unwrap()
    }

    pub fn compiler(&self) -> &String {
        &self.compiler
    }

    #[cfg(feature = "quikc-nightly")]
    pub fn append_args(&self) -> &Option<bool> {
        &self.append_args
    }
}

impl Linker
{
    pub fn args(&self) -> Option<&Vec<String>> {
        self.args.as_ref()
    }

    pub fn libraries(&self) -> Option<&Vec<String>> {
        self.libraries.as_ref()
    }

    #[cfg(feature = "quikc-nightly")]
    pub fn append_args(&self) -> &Option<bool> {
        &self.append_args
    }
}