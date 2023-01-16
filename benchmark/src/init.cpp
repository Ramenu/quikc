#include "mcvk/window.hpp"
#include "mcvk/logger.hpp"
#include "mcvk/global.hpp"
#include "mcvk/device.hpp"
#include "mcvk/vkcomponents.hpp"
#include "mcvk/validationlayers.hpp"
#include "mcvk/swapchain.hpp"
#include <vulkan/vulkan.h>
#include <cstring>
#include <cstdlib>
#include <vector>


static void game();
static void init_vulkan(const VkComponents &components, 
                        Device::LogicalDevice &device, 
                        Swapchain &swapchain,
                        GLFWwindow *window) noexcept;
#ifndef NDEBUG
    static bool has_validation_layer_support() noexcept;
#endif

int main() 
{
    // Before initializing the game, check if validation layers are supported
    // (only necessary for debug builds)
    #ifndef NDEBUG
        if (!has_validation_layer_support())
            Logger::fatal_error("Validation layers requested, but not available");
    #endif
    glfwInit();
    game();
    glfwTerminate();
    return 0;
}


static void game()
{
    static constexpr unsigned WIDTH {500}, HEIGHT {500};
    Window window {WIDTH, HEIGHT, "Minecraft"};

    #ifndef NDEBUG
        static constexpr bool USE_DEBUG_MESSENGER = true;
        VkComponents components {USE_DEBUG_MESSENGER, window.self};
    #else
        VkComponents components {window.self};
    #endif

    Device::LogicalDevice device;
    Swapchain swapchain {};

    // Initialize base vulkan instance, setting up physical/logical devices, debug messengers, swapchain, etc.
    init_vulkan(components, device, swapchain, window.self);


    while (!glfwWindowShouldClose(window.self)) [[likely]] {
        glfwPollEvents();
    }

}

// Components must be initialized before this is called, as it can affect the physical device selection
static void init_vulkan(const VkComponents &components, 
                        Device::LogicalDevice &device, 
                        Swapchain &swapchain,
                        GLFWwindow *window) noexcept
{
    const Device::DeviceInfo device_info {Device::select_physical_device(components, window)};
    device = Device::LogicalDevice{device_info};
    swapchain = Swapchain{device_info.device, 
                          components.get_surface(),
                          window,
                          device_info.queue_family_indices, 
                          device.get()};
}

#ifndef NDEBUG
    static bool has_validation_layer_support() noexcept
    {
        u32 count {};
        vkEnumerateInstanceLayerProperties(&count, nullptr);

        std::vector<VkLayerProperties> layer_properties (count);
        vkEnumerateInstanceLayerProperties(&count, layer_properties.data());

        for (auto validation_layer : VALIDATION_LAYERS) {
            bool found = false;
            for (const auto &properties : layer_properties) {
                if (strcmp(validation_layer, properties.layerName) == 0)  {
                    found = true;
                    break;
                }
            }

            if (!found)
                return false;
        }

        return true;  
    }
#endif
