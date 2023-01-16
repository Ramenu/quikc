![icon](./assets/logo.png)

## What is quikc?

quikc aims to be an efficient, minimalistic, build system with an emphasis on safety for C/C++ programs. Whenever a file is being compiled, quikc 
enables dozens of different warning options (may vary depending on the compiler). Unlike most existing build systems such as Make, Ninja, which require
you to manually specify the flags, quikc enables them by default. Moreover, quikc can be setup with only a few lines. It can also detect if any new source
files have been added.

quikc supports parallel builds natively, and can detect when a unit needs to be recompiled, so you're still getting all of the good stuff as you would
in other build systems. However, if your project has many dependencies and a complicated build setup, then quikc is not for you.

Note that quikc defaults to C17 for C source files and C++20 for C++ source files. However, this can be changed in the configuration file. If you
provide your own arguments to the compiler or linker then the default configuration settings will not be used. Also, if you provide a custom compiler
that is unknown to quikc, the default configuration settings will not be used.

> **Warning**<br>
> I do not recommend using quikc in production environments, as it hasn't been thoroughly tested. The software is in beta. If you are already using
> a different build system, please stick to it.<br>
> <br>
> For newer and smaller projects, quikc will work fine as long as you keep the source files in 'src' and dependencies in 'include'. Newer versions
> will probably remove this restriction.


## Who is this for?

quikc is mainly designed for projects that do not have complicated build setups. If you're looking for a seamless way to get a quick project up and
running with lots warnings enabled then quikc is right for you. Other build systems can be a pain to setup, and can have really long configuration files
just to even get a small project working.

## Who is this not for?

If your project requires each compilation unit to be compiled with its own seperate flags, or has more than one build targets, then quikc is not for you.
Also, since quikc enables various warnings, it is very likely that your code will not compile the first time. This can be an issue for larger codebases.
Fortunately, you can disable the default settings by providing your own flags. Exceptions, virtual methods, and RTTI are also disabled by default if the compiler supports it.

## How to use

If you're creating a new project, simply run the 'quikc-init' script in the directory you would like to create your project in. Or if you'd like to do
it manually, create a 'Build.toml' file in the directory and make sure you have your source files listed in './src' and headers listed in './include' 
otherwise the program will not work as intended. 

For a more comprehensive explaination on the configuration file, see the 'Build.toml' file.

## Build instructions

Before building the project, make sure you have git, cargo and the Rust toolchain installed.<br><br>
**On UNIX-like systems:**
  ```console
  user@desktop:~$ git clone https://github.com/Ramenu/quikc && cd ./quikc && chmod +x ./build.sh && ./build.sh
  ```
