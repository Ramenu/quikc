
# generates dependencies for the source files in ./benchmark
from os import system, listdir, chdir
from os.path import join

chdir("./benchmark")
for file in listdir("./src"):
    f = f"./src/{file}"
    system(f"gcc -I./include {f} -MM -o ./buildinfo/deps/{file}.d")