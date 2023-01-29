
// Default build configuration settings

pub const GCC_COMPILER_NONEXCLUSIVE_WARNINGS : [&str; 25] = [
    "-Wall",
    "-Wextra",
    "-Werror",
    "-Wpedantic",
    "-Wimplicit-fallthrough",
    "-Wdouble-promotion",
    "-Wmissing-include-dirs",
    "-Wformat=2",
    "-Wconversion",
    "-Wduplicated-branches",
    "-Wduplicated-cond",
    "-Wshadow",
    "-Wfloat-equal",
    "-Wcast-qual",
    "-Wcast-align",
    "-Wnull-dereference",
    "-Winvalid-pch",
    "-Winline",
    "-Wunsafe-loop-optimizations",
    "-Wlogical-op",
    "-Wuninitialized",
    "-Winvalid-pch",
    "-Wvector-operation-performance",
    "-Wvla",
    "-Wunused"
];


pub const GCC_COMPILER_C_EXCLUSIVE_WARNINGS : [&str; 3] = [
    "-Wwrite-strings",
    "-Wbad-function-cast",
    "-Wjump-misses-init",
];

pub const GCC_COMPILER_CPP_EXCLUSIVE_WARNINGS : [&str; 13] = [
    "-Wctor-dtor-privacy",
    "-Wnon-virtual-dtor",
    "-Woverloaded-virtual",
    "-Wold-style-cast",
    "-Wuseless-cast",
    "-Wmismatched-tags",
    "-Wextra-semi",
    "-Wsuggest-override",
    "-Wsuggest-final-types",
    "-Wsuggest-final-methods",
    "-Wuseless-cast",
    "-Weffc++",
    "-Wvirtual-inheritance",
];

pub const CLANG_COMPILER_NONEXCLUSIVE_WARNINGS : [&str; 51] = [
    "-Werror",
    "-Wpedantic",
    "-Wall",
    "-Wextra",
    "-Walloca",
    "-Wbad-function-cast",
    "-Wcast-align",
    "-Wcast-qual",
    "-Wconversion",
    "-Wdate-time",
    "-Wdeprecated",
    "-Wdouble-promotion",
    "-Wduplicate-decl-specifier",
    "-Wembedded-directive",
    "-Wempty-translation-unit",
    "-Wenum-conversion",
    "-Wflexible-array-extensions",
    "-Wfloat-equal",
    "-Wfloat-conversion",
    "-Wheader-hygiene",
    "-Wimplicit-fallthrough",
    "-Wkeyword-macro",
    "-Wloop-analysis",
    "-Wmicrosoft",
    "-Wnarrowing",
    "-Wover-aligned",
    "-Wpointer-arith",
    "-Wpoison-system-directories",
    "-Wretained-language-linkage",
    "-Wshadow-all",
    "-Wsign-compare",
    "-Wsign-conversion",
    "-Wsometimes-uninitialized",
    "-Wstring-concatenation",
    "-Wstring-conversion",
    "-Wthread-safety",
    "-Wundefined-internal-type",
    "-Wuninitialized",
    "-Wunreachable-code",
    "-Wvector-conversion",
    "-Wzero-length-array",
    "-Wformat-non-iso",
    "-Wformat=2",
    "-Wgnu",
    "-Wgcc-compat",
    "-Wimplicit-float-conversion",
    "-Wmisleading-indentation",
    "-Wmismatched-tags",
    "-Wmissing-braces",
    "-Winvalid-utf8",
    "-Warray-parameter"
];

pub const GCC_AND_CLANG_DIALECT_OPTIONS : [&str; 4] = [
    "-fPIC",
    "-fdiagnostics-color",
    "-ffunction-sections",
    "-fdata-sections",
];

pub const GCC_AND_CLANG_CPP_DIALECT_OPTIONS : [&str; 1] = [
    "-fstrict-enums"
];

pub const CLANG_COMPILER_CPP_WARNINGS : [&str; 18] = [
    "-Wweak-vtables", 
    "-Wdtor-name",
    "-Wsuper-class-method-mismatch",
    "-Wsuggest-override",
    "-Wsuggest-destructor-override",
    "-Woverloaded-virtual",
    "-Wself-assign-overloaded",
    "-Wself-move",
    "-Weffc++",
    "-Wmissing-noreturn",
    "-Wredundant-move",
    "-Wnon-virtual-dtor",
    "-Wold-style-cast",
    "-Wundefined-reinterpret-cast",
    "-Wsuggest-destructor-override",
    "-Wsuggest-override",
    "-Wsuper-class-method-mismatch",
    "-Winconsistent-missing-destructor-override"
];

pub const GCC_COMPILER_CPP_DIALECT_OPTIONS : [&str; 4] = [
    "-fno-exceptions",
    "-fno-unwind-tables",
    "-fimplicit-constexpr",
    "-fno-rtti"
];

// Static analysis slows down compilation time, but can be disabled
// if explicitly desired
pub const GCC_STATIC_ANALYSIS_OPTIONS : [&str; 1] = [
    "-fanalyzer"
];

pub const GCC_AND_CLANG_OPTIMIZATION_OPTIONS : [&str; 2] = [
    "-O2",
    "-flto"
];

// using this is not recommended as it can potentially cause
// unintentional bugs or crashes with the program
pub const GCC_AND_CLANG_ENHANCED_OPTIMIZATION_OPTIONS : [&str; 4] = [
    "-Ofast",
    "-march=native",
    "-mtune=native",
    "-flto"
];

pub const GCC_PROFILING_OPTIONS : [&str; 1] = [
    "fprofile-use"
];

// Only used for release builds
pub const GCC_AND_CLANG_LINKER_OPTIONS : [&str; 2] = [
    "-flto",
    "-s",
];
