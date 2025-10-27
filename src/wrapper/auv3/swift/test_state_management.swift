#!/usr/bin/env swift

import Foundation
import AVFoundation

// Simple test to verify state management functionality
// This test simulates the Swift fullState property behavior

// Mock the FFI functions for testing
func plugin_create() -> UnsafeMutableRawPointer? {
    // Return a mock handle
    return UnsafeMutableRawPointer(bitPattern: 0x1234)
}

func plugin_destroy(_ handle: UnsafeMutableRawPointer?) -> Int32 {
    return 0
}

func plugin_save_state(_ handle: UnsafeMutableRawPointer?, _ data: UnsafeMutablePointer<UnsafeMutablePointer<UInt8>?>, _ size: UnsafeMutablePointer<UInt32>) -> Int32 {
    // Mock state data
    let stateString = "{\"version\":\"1.0.0\",\"params\":{\"gain\":0.5}}"
    let stateData = stateString.data(using: .utf8)!
    
    let buffer = UnsafeMutablePointer<UInt8>.allocate(capacity: stateData.count)
    stateData.copyBytes(to: buffer, count: stateData.count)
    
    data.pointee = buffer
    size.pointee = UInt32(stateData.count)
    
    return 0
}

func plugin_load_state(_ handle: UnsafeMutableRawPointer?, _ data: UnsafePointer<UInt8>, _ size: UInt32) -> Int32 {
    // Mock state loading
    let stateData = Data(bytes: data, count: Int(size))
    let stateString = String(data: stateData, encoding: .utf8) ?? ""
    print("Loading state: \(stateString)")
    return 0
}

func plugin_free(_ ptr: UnsafeMutableRawPointer?) {
    if let ptr = ptr {
        ptr.deallocate()
    }
}

// Test the fullState property behavior
func testFullStateProperty() {
    print("Testing fullState property...")
    
    // Mock plugin handle
    let handle = plugin_create()
    guard let handle = handle else {
        print("❌ Failed to create plugin handle")
        return
    }
    
    // Test getter
    var data: UnsafeMutablePointer<UInt8>?
    var size: UInt32 = 0
    
    let saveResult = plugin_save_state(handle, &data, &size)
    if saveResult == 0 && data != nil {
        let stateData = Data(bytes: data!, count: Int(size))
        let fullState = ["state": stateData]
        print("✅ State saved successfully: \(fullState)")
        
        // Test setter
        if let savedState = fullState["state"] as? Data {
            let loadResult = savedState.withUnsafeBytes { bytes in
                plugin_load_state(handle, bytes.bindMemory(to: UInt8.self).baseAddress!, UInt32(savedState.count))
            }
            if loadResult == 0 {
                print("✅ State loaded successfully")
            } else {
                print("❌ Failed to load state: \(loadResult)")
            }
        }
        
        // Clean up
        plugin_free(data!)
    } else {
        print("❌ Failed to save state: \(saveResult)")
    }
    
    // Clean up plugin
    plugin_destroy(handle)
}

// Run the test
testFullStateProperty()
print("State management test completed!")