#include "mcvk/vkcomponents.hpp"
#include <GLFW/glfw3.h>
#include <vector>
#include "mcvk/validationlayers.hpp"
#include <cstring>

#ifndef NDEBUG

     // This is just a wrapper around 'vkCreateDebugUtilsMessengerEXT'
    static VkResult CreateDebugUtilsMessengerEXT(VkInstance instance, 
                                                 const VkDebugUtilsMessengerCreateInfoEXT* pCreateInfo, 
                                                 const VkAllocationCallbacks* pAllocator, 
                                                 VkDebugUtilsMessengerEXT* pDebugMessenger) noexcept
    {
        auto func = reinterpret_cast<PFN_vkCreateDebugUtilsMessengerEXT>(vkGetInstanceProcAddr(instance, "vkCreateDebugUtilsMessengerEXT"));
        if (func != nullptr)
            return func(instance, pCreateInfo, pAllocator, pDebugMessenger);
        return VK_ERROR_EXTENSION_NOT_PRESENT;
    }

    

    // Vulkan debugger callback
    static VKAPI_ATTR VkBool32 VKAPI_CALL vk_debug_callback(VkDebugUtilsMessageSeverityFlagBitsEXT severity,
                                                            VkDebugUtilsMessageTypeFlagsEXT,
                                                            const VkDebugUtilsMessengerCallbackDataEXT *p_callback_data,
                                                            void *) noexcept
    {
        // call the appropriate logging function according to the severity level
        switch (severity) {
            default:
                Logger::error("Unknown severity level");
                Logger::error(p_callback_data->pMessage);
                break;
            case VK_DEBUG_UTILS_MESSAGE_SEVERITY_VERBOSE_BIT_EXT: // diagnostic message
                Logger::diagnostic(p_callback_data->pMessage);
                break;
            case VK_DEBUG_UTILS_MESSAGE_SEVERITY_INFO_BIT_EXT: // informational message
                Logger::info(p_callback_data->pMessage);
                break;
            case VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT: // warning (very likely about a bug)
                Logger::warning(p_callback_data->pMessage);
                break;
            case VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT: // error
                Logger::error(p_callback_data->pMessage);
                break;
        }


        /* The callback returns a boolean that indicates if the Vulkan call 
           that triggered the validation layer message should be aborted. If 
           the callback returns true, then the call is aborted with the VK_ERROR_VALIDATION_FAILED_EXT error. 
           So just leave it as VK_FALSE */
        return VK_FALSE;

    }

    VkComponents::VkComponents(bool use_messenger, GLFWwindow *window) noexcept :
        uses_debug_messenger {use_messenger}
    {
        static constexpr VkApplicationInfo app_info {
            .sType = VK_STRUCTURE_TYPE_APPLICATION_INFO,
            .pNext = nullptr,
            .pApplicationName = "Minecraft",
            .applicationVersion = VK_API_VERSION_1_0,
            .pEngineName = "No Engine",
            .engineVersion = VK_API_VERSION_1_0,
            .apiVersion = VK_API_VERSION_1_0,
        };

        static const std::vector<const char*> glfw_extensions = {[use_messenger](){
            uint32_t glfw_extension_count = 0;
            const char **glfw_extensions_ptr = glfwGetRequiredInstanceExtensions(&glfw_extension_count);
            std::vector<const char *> extensions(glfw_extensions_ptr, glfw_extensions_ptr + glfw_extension_count);
            if (use_messenger)
                extensions.push_back(VK_EXT_DEBUG_UTILS_EXTENSION_NAME);
            return extensions;
        }()};

        #ifndef NDEBUG
                static constexpr VkDebugUtilsMessengerCreateInfoEXT DEBUG_CREATE_INFO {
                    .sType = VK_STRUCTURE_TYPE_DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
                    .pNext = nullptr,
                    .flags = 0x0,
                    .messageSeverity = VK_DEBUG_UTILS_MESSAGE_SEVERITY_VERBOSE_BIT_EXT | VK_DEBUG_UTILS_MESSAGE_SEVERITY_INFO_BIT_EXT |
                                       VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT | VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT,
                    .messageType = VK_DEBUG_UTILS_MESSAGE_TYPE_GENERAL_BIT_EXT | VK_DEBUG_UTILS_MESSAGE_TYPE_VALIDATION_BIT_EXT |
                                   VK_DEBUG_UTILS_MESSAGE_TYPE_PERFORMANCE_BIT_EXT,
                    .pfnUserCallback = vk_debug_callback,
                    .pUserData = nullptr
                };
        #endif
        VkInstanceCreateInfo instance_create_info {};
        instance_create_info.sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO;
        instance_create_info.pApplicationInfo = &app_info;
        instance_create_info.enabledExtensionCount = static_cast<u32>(glfw_extensions.size());
        instance_create_info.ppEnabledExtensionNames = glfw_extensions.data();
        #ifndef NDEBUG
            instance_create_info.ppEnabledLayerNames = VALIDATION_LAYERS.data();
            instance_create_info.enabledLayerCount = static_cast<u32>(VALIDATION_LAYERS.size());
            instance_create_info.pNext = &DEBUG_CREATE_INFO;
        #endif

        if (vkCreateInstance(&instance_create_info, nullptr, &instance) != VK_SUCCESS)
            Logger::fatal_error("Failed to initialize vulkan instance");
        if constexpr (Global::IS_DEBUG_BUILD)
            Logger::info("Created vulkan instance successfully");

        #ifndef NDEBUG
            if (use_messenger)
                if (CreateDebugUtilsMessengerEXT(instance, &DEBUG_CREATE_INFO, nullptr, &messenger) != VK_SUCCESS)
                    Logger::fatal_error("Failed to setup debug messenger with instance");
        #endif

        // Create the window surface
        if (glfwCreateWindowSurface(instance, window, nullptr, &surface) != VK_SUCCESS)
            Logger::fatal_error("Failed to create window surface");
        if constexpr (Global::IS_DEBUG_BUILD)
            Logger::info("Created window surface successfully");
    }
#endif


VkComponents::~VkComponents() noexcept
{
    // De-allocate debug messenger
    if constexpr (Global::IS_DEBUG_BUILD) {
        if (messenger != VK_NULL_HANDLE && uses_debug_messenger) {
            Logger::info("De-allocating debug messenger");
            auto func = reinterpret_cast<PFN_vkDestroyDebugUtilsMessengerEXT>(vkGetInstanceProcAddr(instance, "vkDestroyDebugUtilsMessengerEXT"));
            if (func != nullptr) {
                func(instance, messenger, nullptr);
                messenger = VK_NULL_HANDLE;
            }
            else
                Logger::error("Failed to load 'vkDestroyDebugUtilsMessengerEXT' address");
        }
    }

    if (surface != VK_NULL_HANDLE) {
        if constexpr (Global::IS_DEBUG_BUILD)
            Logger::info("De-allocating VkSurfaceKHR");
        vkDestroySurfaceKHR(instance, surface, nullptr);
        surface = VK_NULL_HANDLE;
    }

    if (instance != VK_NULL_HANDLE) {
        if constexpr (Global::IS_DEBUG_BUILD)
            Logger::info("De-allocating VkInstance");
        vkDestroyInstance(instance, nullptr);
        instance = VK_NULL_HANDLE;
    }
}