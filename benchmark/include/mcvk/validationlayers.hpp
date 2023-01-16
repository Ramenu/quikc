#ifndef MCVK_VALIDATIONLAYERS_HPP
#define MCVK_VALIDATIONLAYERS_HPP

#ifndef NDEBUG
    #include <array>
    static constexpr std::array VALIDATION_LAYERS {
        "VK_LAYER_KHRONOS_validation"
    };
#endif

#endif // MCVK_VALIDATIONLAYERS_HPP
