#ifndef MCVK_PHYSICALDEVICEINFO_HPP
#define MCVK_PHYSICALDEVICEINFO_HPP

#include <vulkan/vulkan.h>
#ifndef NDEBUG
    #include <string>
#endif

namespace Device
{
    struct PhysicalDeviceInfo
    {
        VkPhysicalDevice self {};
        #ifndef NDEBUG
            std::string name {};
        #endif
    };
}

#endif // MCVK_PHYSICALDEVICEINFO_HPP