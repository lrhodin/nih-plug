#include <stdio.h>
#include <AudioToolbox/AudioToolbox.h>

int main() {
    printf("Testing AUv3 plugin discovery...\n");
    
    // Try to find the plugin using AudioComponentFindNext
    AudioComponentDescription desc = {
        .componentType = kAudioUnitType_Effect,
        .componentSubType = 0x6E706C67, // 'nplg'
        .componentManufacturer = 0x4E504C47, // 'NPLG'
        .componentFlags = 0,
        .componentFlagsMask = 0
    };
    
    AudioComponent component = AudioComponentFindNext(NULL, &desc);
    if (component != NULL) {
        printf("Found Audio Unit component!\n");
        
        // Try to get component info
        CFStringRef name;
        UInt32 size = sizeof(name);
        OSStatus status = AudioComponentGetDescription(component, &desc);
        if (status == noErr) {
            printf("Component description retrieved successfully\n");
            printf("Type: %c%c%c%c\n", 
                   (desc.componentType >> 24) & 0xFF,
                   (desc.componentType >> 16) & 0xFF,
                   (desc.componentType >> 8) & 0xFF,
                   desc.componentType & 0xFF);
            printf("Subtype: %c%c%c%c\n", 
                   (desc.componentSubType >> 24) & 0xFF,
                   (desc.componentSubType >> 16) & 0xFF,
                   (desc.componentSubType >> 8) & 0xFF,
                   desc.componentSubType & 0xFF);
            printf("Manufacturer: %c%c%c%c\n", 
                   (desc.componentManufacturer >> 24) & 0xFF,
                   (desc.componentManufacturer >> 16) & 0xFF,
                   (desc.componentManufacturer >> 8) & 0xFF,
                   desc.componentManufacturer & 0xFF);
        } else {
            printf("Failed to get component description: %d\n", (int)status);
        }
    } else {
        printf("Audio Unit component not found\n");
    }
    
    return 0;
}