use std::{time::Instant, env, path::Path};

use color_print::cprintln;

use crate::{build::Build, walker, SOURCE_DIRECTORY, buildtable::{self, BUILD_TABLE_DIRECTORY, BuildTable}, compiler, test::{initialize_project, Settings, Tools, modify_file_time}};

const SAMPLES : usize = 1000;

fn print_benchmark_results(task_msg : &str, mean : f64, std : f64)
{
    cprintln!("<red><bold>({})</bold></red> {} 'mean': <green>{} nanoseconds</green>", SAMPLES, task_msg, mean);
    cprintln!("<red><bold>({})</bold></red> {} 'std': <green>{:.5}</green>\n", SAMPLES, task_msg, std);
}

fn benchmark_fn<T>(task_msg : &str, f : &mut T) 
    //where T : FnMut() -> Result<(), Box<dyn std::error::Error>>
    where T : FnMut() -> ()
{
    let mut v = Vec::new();

    for _ in 0..SAMPLES {
        let start = Instant::now();
        f();
        let duration = start.elapsed().as_nanos() as f64;
        v.push(duration);
    }

    let mean = statistical::mean(v.as_slice());
    let std = statistical::standard_deviation(v.as_slice(), Some(mean)) / 1000.0;

    print_benchmark_results(task_msg, mean, std);

}

fn reset() -> Result<(), Box<dyn std::error::Error>>
{
    if Path::new(BUILD_TABLE_DIRECTORY).exists() {
        std::fs::remove_dir_all(BUILD_TABLE_DIRECTORY)?;
    }
    Ok(())
}

#[test]
fn quikc_benchmark() -> Result<(), Box<dyn std::error::Error>>
{
    const BENCHMARK_DIR : &str = "./benchmark";

    // cd into benchmark directory and remove the build table directory so we can recompile
    // from scratch
    env::set_current_dir(BENCHMARK_DIR)?;
    reset()?;

    benchmark_fn("time to initialize build configuration", &mut || {Build::new();});
    benchmark_fn("time to initialize build table", &mut || {BuildTable::new(&mut toml::value::Table::new());});


    // Benchmark first time retrieving source file speed
    {
        let mut tools = Tools::new();
        benchmark_fn("time to retrieve source files on first compilation",&mut ||walker::retrieve_source_files(SOURCE_DIRECTORY, 
                                &mut tools.source_files, 
                                &tools.build_config.get_compiler_name(), 
                                &mut tools.build_table,
                                &mut tools.old_table));
    }

    // Benchmark retrieving source files speed when a dependency has changed
    {
        let mut tools = Tools::new();
        modify_file_time("./include/mcvk/device.hpp")?;
        benchmark_fn("time to retrieve source files on header file change",&mut ||walker::retrieve_source_files(SOURCE_DIRECTORY, 
                                &mut tools.source_files, 
                                &tools.build_config.get_compiler_name(), 
                                &mut tools.build_table,
                                &mut tools.old_table));
    }


    Ok(())
}