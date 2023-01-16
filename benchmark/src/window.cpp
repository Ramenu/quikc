#include "mcvk/window.hpp"
#ifndef NDEBUG
    #include "mcvk/logger.hpp"
    #include "mcvk/global.hpp"
#endif

Window::Window(int wwidth, int wheight, const char *wname) noexcept :
    width {wwidth}, height {wheight}, 
    self {[wwidth, wheight, wname](){
        glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API);
        glfwWindowHint(GLFW_RESIZABLE, GLFW_FALSE);
        return glfwCreateWindow(wwidth, wheight, wname, nullptr, nullptr);
    }()}
{
}

Window::~Window() noexcept
{
    if (self != nullptr) {
        if constexpr (Global::IS_DEBUG_BUILD)
            Logger::info("De-allocating window");
        glfwDestroyWindow(self);
        self = nullptr;
    }
}

void Window::create_surface(VkInstance instance, GLFWwindow &window, VkSurfaceKHR surface) noexcept
{
    if (glfwCreateWindowSurface(instance, &window, nullptr, &surface) != VK_SUCCESS) 
        Logger::fatal_error("Failed to create window surface");
    if constexpr (Global::IS_DEBUG_BUILD)
        Logger::info("Window surface created successfully");
}
