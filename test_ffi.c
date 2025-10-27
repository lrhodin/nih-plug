#include <stdio.h>
#include <stdlib.h>
#include <dlfcn.h>

// Function pointer types
typedef void* (*plugin_create_t)(void);
typedef int32_t (*plugin_destroy_t)(void*);
typedef int32_t (*plugin_initialize_t)(void*, float, uint32_t, uint32_t, uint32_t);

int main() {
    printf("Testing NIH-Plug AUv3 FFI functions...\n");
    
    // Load the dynamic library
    void* handle = dlopen("./target/bundled/gain.appex/Contents/MacOS/gain", RTLD_LAZY);
    if (!handle) {
        printf("Error loading library: %s\n", dlerror());
        return 1;
    }
    
    // Get function pointers
    plugin_create_t plugin_create = (plugin_create_t)dlsym(handle, "plugin_create");
    plugin_destroy_t plugin_destroy = (plugin_destroy_t)dlsym(handle, "plugin_destroy");
    plugin_initialize_t plugin_initialize = (plugin_initialize_t)dlsym(handle, "plugin_initialize");
    
    if (!plugin_create || !plugin_destroy || !plugin_initialize) {
        printf("Error getting function symbols: %s\n", dlerror());
        dlclose(handle);
        return 1;
    }
    
    printf("Successfully loaded FFI functions\n");
    
    // Test plugin creation
    void* plugin = plugin_create();
    if (!plugin) {
        printf("Error: plugin_create returned NULL\n");
        dlclose(handle);
        return 1;
    }
    
    printf("Successfully created plugin instance\n");
    
    // Test plugin initialization
    int32_t result = plugin_initialize(plugin, 44100.0f, 512, 2, 2);
    if (result != 0) {
        printf("Error: plugin_initialize failed with code %d\n", result);
        plugin_destroy(plugin);
        dlclose(handle);
        return 1;
    }
    
    printf("Successfully initialized plugin\n");
    
    // Clean up
    plugin_destroy(plugin);
    dlclose(handle);
    
    printf("FFI test completed successfully!\n");
    return 0;
}