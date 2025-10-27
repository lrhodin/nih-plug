#include <AudioUnit/AudioUnit.h>
#include <AudioToolbox/AudioToolbox.h>
#include <stdio.h>
#include <stdlib.h>

int main() {
    printf("Testing AUv3 plugin loading...\n");
    
    // Try to find the plugin
    AudioComponentDescription desc = {0};
    desc.componentType = 'aumu';
    desc.componentSubType = 'nplg';
    desc.componentManufacturer = 'NPLG';
    desc.componentFlags = 0;
    desc.componentFlagsMask = 0;
    
    AudioComponent component = AudioComponentFindNext(NULL, &desc);
    if (component == NULL) {
        printf("ERROR: Plugin not found\n");
        return 1;
    }
    
    printf("Plugin found!\n");
    
    // Try to open the plugin
    AudioUnit audioUnit = NULL;
    OSStatus status = AudioComponentInstanceNew(component, &audioUnit);
    if (status != noErr) {
        printf("ERROR: Failed to create AudioUnit instance: %d\n", (int)status);
        return 1;
    }
    
    printf("Plugin loaded successfully!\n");
    
    // Clean up
    AudioComponentInstanceDispose(audioUnit);
    
    return 0;
}