#include <cstdio>
#include <cstdlib>
#include <mutex>
#include "mcvk/color.h"


namespace Logger
{

    static std::mutex mtx;

    // NOLINTBEGIN(cppcoreguidelines-pro-type-vararg)
    [[noreturn]] void fatal_error(const char *msg) noexcept
    {
        // Need a lock here because exit is not thread-safe, it uses a
        // global variable which is not protected so a race condition can
        // occur. 
        std::lock_guard lock {mtx};
        fprintf(stderr, COLOR_BOLDRED "FATAL ERROR: " COLOR_RESET "%s\n", msg);
        exit(EXIT_FAILURE);
    }

    void error(const char *msg) noexcept 
    {
        fprintf(stderr, COLOR_RED "ERROR: " COLOR_RESET "%s\n", msg);
    }

    #ifndef NDEBUG
        void diagnostic(const char *msg) noexcept
        {
            fprintf(stdout, COLOR_MAGENTA "DIAGNOSTIC: " COLOR_RESET "%s\n", msg);
        }

        void info(const char *msg) noexcept
        {
            fprintf(stdout, COLOR_MAGENTA "INFO: " COLOR_RESET "%s\n", msg);
        }

        void warning(const char *msg) noexcept
        {
            fprintf(stderr, COLOR_YELLOW "WARNING: " COLOR_RESET "%s\n", msg);
        }
    #endif
    // NOLINTEND(cppcoreguidelines-pro-type-vararg)
}