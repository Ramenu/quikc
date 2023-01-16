 #include "mcvk/device.hpp"
 #include "mcvk/logger.hpp"
 #include "mcvk/global.hpp"
 #include "mcvk/swapchain.hpp"
 #include <vector>
 #include <string>
 #include <set>
 #include <array>
 #include <cstring>


namespace Device
{

    static constexpr std::array REQUIRED_DEVICE_EXTENSIONS {
        VK_KHR_SWAPCHAIN_EXTENSION_NAME // Not all GPUs can present images to a screen so this is required
    };

    static unsigned device_type_rating(VkPhysicalDeviceType type) noexcept
    {
        switch (type)
        {
            case VK_PHYSICAL_DEVICE_TYPE_DISCRETE_GPU: return 2;
            case VK_PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU: return 1;
            // other types also exist, but for simplicity sake just
            // return 0.
            default: return 0;
        }
    }

    static bool received_vram_retrieval_error(const DeviceInfo &info)
    {
        if (!(info.memory_heap.flags&VkMemoryHeapFlagBits::VK_MEMORY_HEAP_DEVICE_LOCAL_BIT)) {
            const auto msg = std::string{"Failed to retrieve "} + info.properties.deviceName + " VRAM size";
            Logger::error(msg.c_str());
            return true;
        }
        return false;
    }

    // keep in mind this comparsion is quite primitive, but it should at least be good enough
    static Global::Compare compare_device_specs(const DeviceInfo &one, const DeviceInfo &two)
    {
        // in the future can also check for sparse binding support, dynamic array indexing, and tesselation support,
        u32 score_one {}, score_two {};
        const auto device_type_one = device_type_rating(one.properties.deviceType);
        const auto device_type_two = device_type_rating(two.properties.deviceType);

        // Compare GPU type (dedicated VS integrated)
        if (device_type_one > device_type_two)
            score_one += 2;
        else if (device_type_one < device_type_two)
            score_two += 2;
        
        if (received_vram_retrieval_error(one) || received_vram_retrieval_error(two))
            return Global::Compare::Fail;
        
        // Compare VRAM size
        if (one.memory_heap.size > two.memory_heap.size)
            ++score_one;
        else if (one.memory_heap.size < two.memory_heap.size)
            ++score_two;
        
        // Compare maximum 2D texture size (higher values make for better quality)
        if (one.properties.limits.maxImageDimension2D > two.properties.limits.maxImageDimension2D)
            ++score_one;
        else if (one.properties.limits.maxImageDimension2D < two.properties.limits.maxImageDimension2D)
            ++score_two;
        
        if (score_one > score_two)
            return Global::Compare::Greater;
        else if (score_one < score_two)
            return Global::Compare::Less;
        return Global::Compare::Equal;
    }

    static bool device_has_extension_support(const DeviceInfo &info) noexcept
    {
        u32 extension_count {};
        vkEnumerateDeviceExtensionProperties(info.device.self, nullptr, &extension_count, nullptr);

        if (extension_count == 0) {
            if constexpr (Global::IS_DEBUG_BUILD) {
                const auto msg = std::string{"No extensions found for device "} + info.properties.deviceName;
                Logger::info(msg.c_str());
            }
            return false;
        }

        std::vector<VkExtensionProperties> available_device_extensions (extension_count);
        vkEnumerateDeviceExtensionProperties(info.device.self, nullptr, &extension_count, available_device_extensions.data());

        std::set<std::string> required_extensions {REQUIRED_DEVICE_EXTENSIONS.begin(), REQUIRED_DEVICE_EXTENSIONS.end()};

        for (const auto &extension : available_device_extensions) {
            if constexpr (Global::IS_DEBUG_BUILD) {
                if (required_extensions.find(extension.extensionName) != required_extensions.end()) {
                    const auto msg = std::string{"Device "} + info.properties.deviceName + " supports " + extension.extensionName;
                    Logger::info(msg.c_str());
                }
            }
            required_extensions.erase(extension.extensionName);
        }

        if constexpr (Global::IS_DEBUG_BUILD) {

            // print out any extensions that were not found
            for (const auto &extension : required_extensions) {
                const auto msg = std::string{"Device "} + info.properties.deviceName + " does not support " + extension;
                Logger::info(msg.c_str());
            }

            if (!required_extensions.empty()) {
                const auto msg = std::string{"Device "} + info.properties.deviceName + " does not support required extensions";
                Logger::info(msg.c_str());
            }
            else {
                const auto msg = std::string{"Device "} + info.properties.deviceName + " supports required extensions";
                Logger::info(msg.c_str());
            }
        }
        return required_extensions.empty();
    }

    static bool can_use_physical_device(const DeviceInfo &info, const Swapchain &swapchain) noexcept
    {
        const bool extensions_supported = device_has_extension_support(info);

        if (extensions_supported) {
            return info.features.geometryShader && info.queue_family_indices.is_complete() &&
                   swapchain.is_compatible();
        }
        return false;
    }

    [[nodiscard]] DeviceInfo select_physical_device(const VkComponents &components, GLFWwindow *window) noexcept
    {
        u32 count {};
        vkEnumeratePhysicalDevices(components.get_instance(), &count, nullptr);
        std::vector<VkPhysicalDevice> devices (count);

        if (count == 0)
            Logger::fatal_error("Could not find available GPUs with Vulkan support");
        
        vkEnumeratePhysicalDevices(components.get_instance(), &count, devices.data());
        
        DeviceInfo previous_device_info {};
        DeviceInfo selected_device_info {};
        bool appropriate_device_exists = false;

        // iterate through all the available devices in the
        // system and try to select the best one
        for (auto device : devices) {
            DeviceInfo info {};
            info.device.self = device;
            VkPhysicalDeviceMemoryProperties device_mem_properties {};

            vkGetPhysicalDeviceProperties(device, &info.properties);
            vkGetPhysicalDeviceMemoryProperties(device, &device_mem_properties);
            vkGetPhysicalDeviceFeatures(device, &info.features);

            info.device.name = info.properties.deviceName;
            #ifndef NDEBUG
                const auto check_dev_msg = std::string{"Checking device: "} + info.device.name;
                Logger::info(check_dev_msg.c_str());
            #endif

            const auto memory_heaps_ptr {device_mem_properties.memoryHeaps};
            const std::vector<VkMemoryHeap> memory_heaps {memory_heaps_ptr, memory_heaps_ptr + device_mem_properties.memoryHeapCount};

            // find VRAM size
            for (auto heap : memory_heaps) {
                if (heap.flags&VkMemoryHeapFlagBits::VK_MEMORY_HEAP_DEVICE_LOCAL_BIT) {
                    info.memory_heap = heap;
                    break;
                }
            }
            #ifndef NDEBUG
                const Device::PhysicalDeviceInfo physical_device_info {
                    .self = device,
                    .name = info.properties.deviceName
                };
            #else
                const Device::PhysicalDeviceIfo physical_device_info {
                    .self = device
                };
            #endif

            info.queue_family_indices = Queue::QueueFamilyIndices{physical_device_info, components.get_surface()};
            const Swapchain swapchain {info.device, components.get_surface(), window, info.queue_family_indices, VK_NULL_HANDLE};

            const bool can_use_device = can_use_physical_device(info, swapchain);

            // device must be compatible in order to use it
            if (can_use_device) {
                if constexpr (Global::IS_DEBUG_BUILD) {
                    const auto msg = std::string{"Device "} + info.properties.deviceName + " supports all required features.";
                    Logger::info(msg.c_str());
                }
                // if the previous device wasnt initialized yet, set the selected device to this device
                if (previous_device_info.device.self == VK_NULL_HANDLE) {
                    appropriate_device_exists = true;
                    selected_device_info.device = info.device;
                    selected_device_info.properties = info.properties;
                    selected_device_info.features = info.features;
                    selected_device_info.memory_heap = info.memory_heap;
                    selected_device_info.queue_family_indices = info.queue_family_indices;
                }
                else {
                    // now we can actually compare the devices
                    const auto cmp = compare_device_specs(info, previous_device_info);
                    if (cmp == Global::Compare::Greater) {
                        selected_device_info.device = info.device;
                        selected_device_info.properties = info.properties;
                        selected_device_info.features = info.features;
                        selected_device_info.memory_heap = info.memory_heap;
                        selected_device_info.queue_family_indices = info.queue_family_indices;
                    }
                }
                previous_device_info = std::move(info);
            }
            #ifndef NDEBUG 
                else {
                    const auto msg = std::string{"Device "} + info.properties.deviceName + " does not support required features. Skipping...";
                    Logger::info(msg.c_str());
                }
            #endif

        }

        if (!appropriate_device_exists)
            Logger::fatal_error("Could not find a suitable GPU to run the game");

        #ifndef NDEBUG
            const auto msg = std::string{"Selected physical device: "} + selected_device_info.properties.deviceName;
            Logger::info(msg.c_str());
        #endif

        return selected_device_info;
    }

    LogicalDevice::LogicalDevice(const DeviceInfo &selected_device_info) noexcept
    {
        constexpr float QUEUE_PRIORITY {1.0f};

        if (!selected_device_info.queue_family_indices.is_complete())
            Logger::fatal_error("Selected device should have all required queue families. If you're seeing this error, report this as a bug.");

        std::vector<VkDeviceQueueCreateInfo> queue_create_infos {};
        const std::set<u32> unique_queue_families {selected_device_info.queue_family_indices.array().begin(), 
                                                   selected_device_info.queue_family_indices.array().end()};

        for (auto queue_family : unique_queue_families) {
            VkDeviceQueueCreateInfo queue_create_info {};
            queue_create_info.sType = VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO;
            queue_create_info.queueFamilyIndex = queue_family;
            queue_create_info.queueCount = 1;

            // set priority of queue to influence scheduling of command buffer
            queue_create_info.pQueuePriorities = &QUEUE_PRIORITY;
            queue_create_infos.push_back(queue_create_info);
        }

        VkDeviceCreateInfo device_create_info {};
        device_create_info.sType = VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO;
        device_create_info.pQueueCreateInfos = queue_create_infos.data();
        device_create_info.queueCreateInfoCount = static_cast<u32>(queue_create_infos.size());
        device_create_info.pEnabledFeatures = &selected_device_info.features;
        device_create_info.enabledExtensionCount = static_cast<u32>(REQUIRED_DEVICE_EXTENSIONS.size());
        device_create_info.ppEnabledExtensionNames = REQUIRED_DEVICE_EXTENSIONS.data();

        if (vkCreateDevice(selected_device_info.device.self, &device_create_info, nullptr, &device) != VK_SUCCESS)
            Logger::fatal_error("Failed to create logical device");
        #ifndef NDEBUG
            else
                Logger::info("Logical device created successfully");
        #endif

        devices_in_use.insert(device); // We are now using the device so add it to the set

        vkGetDeviceQueue(device, 
                         selected_device_info.queue_family_indices.get(Queue::GraphicsQueueIndex), 
                         0, 
                         &graphics_queue);

        vkGetDeviceQueue(device, 
                         selected_device_info.queue_family_indices.get(Queue::PresentationQueueIndex), 
                         0, 
                         &presentation_queue);
    }

    LogicalDevice::~LogicalDevice() noexcept
    {
        if (device != VK_NULL_HANDLE) {
            if constexpr (Global::IS_DEBUG_BUILD) {
                Logger::info("De-allocating logical device");
                if (!this->device_is_in_use(device)) {
                    Logger::fatal_error("Attempted to de-allocate logical device, but it is not being used. Fix this bug");
                }
            }
            vkDestroyDevice(device, nullptr); 
            devices_in_use.erase(device); // No longer using the device so erase it
            device = VK_NULL_HANDLE;
        }
    }


}
