use std::{time::Instant, env, path::Path, fs::{File, self}, io::Write};

use color_print::cprintln;
use once_cell::sync::Lazy;

use crate::{build::Build, walker, SOURCE_DIRECTORY, buildtable::{self, BUILD_TABLE_DIRECTORY, BuildTable}, compiler, test::{initialize_project, Settings, Tools, modify_file_time}};

const SAMPLES : usize = 10000;
const BENCHMARK_LOG_FILE_PATH : &str = "../benchmark.log";
const OLD_LOG_FILE_PATH : &str = "../old-benchmark.log";
const LOG_DIRECTORY : &str = "../logs";

static mut BENCHMARK_LOG_FILE : once_cell::sync::Lazy<File> = Lazy::new(|| {
    let benchmark_log_file_exists = Path::new(BENCHMARK_LOG_FILE_PATH).is_file();
    if benchmark_log_file_exists {
        fs::copy(BENCHMARK_LOG_FILE_PATH, OLD_LOG_FILE_PATH).expect("Failed to copy from log to old log file");
    }

    if !Path::new(LOG_DIRECTORY).is_dir() {
        fs::create_dir(LOG_DIRECTORY).expect("Failed to create log directory");
    }

    let paths = fs::read_dir(LOG_DIRECTORY).expect("Failed to read log directory");
    let hi = paths.count() + 1;
    
    if benchmark_log_file_exists {
        fs::copy(BENCHMARK_LOG_FILE_PATH, format!("{}/old-benchmark{}.log", LOG_DIRECTORY, hi)).unwrap();
    }
    return File::create(BENCHMARK_LOG_FILE_PATH).expect("Failed to create/open benchmark log file");
});

fn print_benchmark_results(task_msg : &str, mean : f64, std : f64)
{
    unsafe {
        BENCHMARK_LOG_FILE.write(format!("({}) {} 'mean': {} milliseconds\n", SAMPLES, task_msg, mean).as_bytes()).unwrap();
        BENCHMARK_LOG_FILE.write(format!("({}) {} 'std': {:.5}\n\n", SAMPLES, task_msg, std).as_bytes()).unwrap();
    }
}

fn benchmark_fn<T>(task_msg : &str, f : &mut T) 
    where T : FnMut() -> ()
{
    let mut v = Vec::new();

    for _ in 0..SAMPLES {
        let start = Instant::now();
        f();
        let duration = start.elapsed().as_millis() as f64;
        v.push(duration);
    }

    let mean = statistical::mean(v.as_slice());
    let std = statistical::standard_deviation(v.as_slice(), Some(mean));

    print_benchmark_results(task_msg, mean, std);
}

fn reset() -> Result<(), Box<dyn std::error::Error>>
{
    if Path::new(BUILD_TABLE_DIRECTORY).exists() {
        std::fs::remove_dir_all(BUILD_TABLE_DIRECTORY)?;
    }
    Ok(())
}

fn compare_benchmarks(file_name : &str) -> Result<(), Box<dyn std::error::Error>>
{
    let mean_reg = regex::Regex::new(r"\(\d+\) ((?:(?:\s|\w+)+|\s) 'mean'): ((?:\d|\.)+) milliseconds")?;
    let std_reg = regex::Regex::new(r"\(\d+\) ((?:(?:\s|\w+)+|\s) 'std'): ((?:\d|\.)+)")?;

    let old_log_file_as_str = fs::read_to_string(if file_name == "latest" { OLD_LOG_FILE_PATH } else { file_name })?;
    let new_log_file_as_str = fs::read_to_string(BENCHMARK_LOG_FILE_PATH)?;

    let mut old_log = old_log_file_as_str.lines();
    let mut new_log = new_log_file_as_str.lines();

    while let Some(new_line) = new_log.next() {
        let old_new_line = old_log.next().unwrap();
        if !new_line.is_empty() {

            let task_msg_mean = mean_reg.captures(new_line)
                                    .unwrap()
                                    .get(1)
                                    .unwrap()
                                    .as_str();

            let mean = mean_reg.captures(new_line)
                                    .unwrap()
                                    .get(2)
                                    .unwrap()
                                    .as_str()
                                    .parse::<f64>()
                                    .unwrap();

            let next = new_log.next().unwrap();
            let task_msg_std = std_reg.captures(next)
                                    .unwrap()
                                    .get(1)
                                    .unwrap()
                                    .as_str();

            let std = std_reg.captures(next)
                                  .unwrap()
                                  .get(2)
                                  .unwrap()
                                  .as_str()
                                  .parse::<f64>()
                                  .unwrap();


            let old_mean = mean_reg.captures(old_new_line)
                                    .unwrap()
                                    .get(2)
                                    .unwrap()
                                    .as_str()
                                    .parse::<f64>()
                                    .unwrap();

            let old_std = std_reg.captures(old_log.next().unwrap())
                                  .unwrap()
                                  .get(2)
                                  .unwrap()
                                  .as_str()
                                  .parse::<f64>()
                                  .unwrap();

            let mean_diff = mean - old_mean;
            let std_diff = std - old_std;

            print_diff(task_msg_mean, mean_diff, "ms");
            print_diff(task_msg_std, std_diff, "");
            println!("");
            
        }
    }

    Ok(())
}

fn print_diff(msg : &str, diff : f64, unit : &str)
{
    if diff > 0.0 {
        cprintln!("<bold>{}: <red>+{:.3}{}</red></bold>", msg, diff, unit);
    }
    else if diff < 0.0 {
        cprintln!("<bold>{}: <green>{:.3}{}</green></bold>", msg, diff, unit);
    }
    else {
        cprintln!("<bold>{}: <yellow>{:.3}{}</yellow></bold>", msg, diff, unit);
    }
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

    compare_benchmarks("latest")?;

    Ok(())
}