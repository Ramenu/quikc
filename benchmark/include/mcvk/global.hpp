#ifndef MCVK_GLOBAL_HPP
#define MCVK_GLOBAL_HPP

#include "mcvk/types.hpp"

#define DELETE_NON_MOVABLE_DEFAULT(T) \
T &operator=(T &&) = delete; \
T(T &&) = delete;

#define DELETE_NON_COPYABLE_DEFAULT(T) \
T &operator=(const T &) = delete; \
T(const T &) = delete;

#define DELETE_NON_COPYABLE_NON_MOVABLE_DEFAULT(T) \
DELETE_NON_COPYABLE_DEFAULT(T) \
DELETE_NON_MOVABLE_DEFAULT(T)

namespace Global
{
    consteval auto FLAG_SUM(usize max)
    {
        return (1 << (max - 1)) * 2 - 1;
    }
    // Should be used for comparsions between two values/objects.
    // Order should be:
    // $1 > $2: Greater
    // $1 = $2: Equal
    // $1 < $2: Less
    // If a comparsion fails for whatever reason, use Fail
    enum class Compare
    {
        Greater,
        Equal,
        Less,
        Fail
    };

    #ifndef NDEBUG
        static constexpr bool IS_DEBUG_BUILD = true;
    #else
        static constexpr bool IS_DEBUG_BUILD = false;
    #endif

}

#endif // MCVK_GLOBAL_HPP
