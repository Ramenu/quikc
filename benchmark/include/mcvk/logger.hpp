#ifndef MCVK_LOGGER_HPP
#define MCVK_LOGGER_HPP

#include "types.hpp"

namespace Logger
{
    // for unrecoverable errors
    [[noreturn]] extern void fatal_error(const char *msg) noexcept;

    // for recoverable errors
    extern void error(const char *msg) noexcept;


    #ifndef NDEBUG
        // does not serve any purpose other than being used to differentiate logging 
        // and diagnostic messages in the vk debug messenger
        extern void diagnostic(const char *msg) noexcept;

        // for logging messages, debugging, etc.
        extern void info(const char *msg) noexcept;

        // for warnings
        extern void warning(const char *msg) noexcept;
        
    #endif

}

#endif // MCVK_LOGGER_HPP
