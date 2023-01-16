#ifndef MCVK_DEVICE_HPP
#define MCVK_DEVICE_HPP

#include <vulkan/vulkan.h>
#include "mcvk/queue.hpp"
#include "mcvk/vkcomponents.hpp"
#include <GLFW/glfw3.h>
#ifndef NDEBUG
    #include <string>
#endif
#include <set>

namespace Device
{
    struct DeviceInfo
    {
        PhysicalDeviceInfo device {};
        VkPhysicalDeviceProperties properties {};
        VkPhysicalDeviceFeatures features {};
        VkMemoryHeap memory_heap {};
        Queue::QueueFamilyIndices queue_family_indices {};
    };

    class LogicalDevice
    {
        private:
            inline static std::set<VkDevice> devices_in_use {};
            VkDevice device {VK_NULL_HANDLE};
            VkQueue graphics_queue {};
            VkQueue presentation_queue {};
        public:
            constexpr LogicalDevice() noexcept = default;
            explicit LogicalDevice(const DeviceInfo &selected_device_info) noexcept;
            constexpr LogicalDevice& operator=(LogicalDevice &&other) noexcept
            {
                this->device = other.device;
                this->graphics_queue = other.graphics_queue;

                other.device = VK_NULL_HANDLE;
                return *this;
            }
            constexpr explicit LogicalDevice(LogicalDevice &&other) noexcept
            {
                this->device = other.device;
                this->graphics_queue = other.graphics_queue;

                other.device = VK_NULL_HANDLE;
            }
            DELETE_NON_COPYABLE_DEFAULT(LogicalDevice)

            ~LogicalDevice() noexcept; 
            constexpr auto get() const { return device; }
            static auto device_is_in_use(VkDevice device) noexcept
            {
                return devices_in_use.find(device) != devices_in_use.end();
            }
    };
    
    extern DeviceInfo select_physical_device(const VkComponents &components, GLFWwindow *window) noexcept;
}

#endif // MCVK_DEVICE_HPP
