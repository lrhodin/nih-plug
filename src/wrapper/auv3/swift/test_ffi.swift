import Foundation
import NIHPlugAUv3

// Test FFI integration
let testHandle = plugin_create()
if testHandle != nil {
    print("FFI plugin_create() succeeded - handle: \(testHandle!)")
    let paramCount = plugin_get_parameter_count(testHandle!)
    print("Plugin has \(paramCount) parameters")
    let destroyResult = plugin_destroy(testHandle!)
    print("Plugin destroy result: \(destroyResult)")
} else {
    print("FFI plugin_create() failed")
}
