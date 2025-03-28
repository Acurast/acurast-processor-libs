mkdir ./AcurastSDK/AcurastSDKFFI.xcframework/ios-arm64/Headers/AcurastSDK
mkdir ./AcurastSDK/AcurastSDKFFI.xcframework/ios-arm64_x86_64-simulator/Headers/AcurastSDK

mv ./AcurastSDK/AcurastSDKFFI.xcframework/ios-arm64/Headers/acurastsdkFFI.h ./AcurastSDK/AcurastSDKFFI.xcframework/ios-arm64/Headers/AcurastSDK/
mv ./AcurastSDK/AcurastSDKFFI.xcframework/ios-arm64/Headers/module.modulemap ./AcurastSDK/AcurastSDKFFI.xcframework/ios-arm64/Headers/AcurastSDK/

mv ./AcurastSDK/AcurastSDKFFI.xcframework/ios-arm64_x86_64-simulator/Headers/acurastsdkFFI.h ./AcurastSDK/AcurastSDKFFI.xcframework/ios-arm64_x86_64-simulator/Headers/AcurastSDK/
mv ./AcurastSDK/AcurastSDKFFI.xcframework/ios-arm64_x86_64-simulator/Headers/module.modulemap ./AcurastSDK/AcurastSDKFFI.xcframework/ios-arm64_x86_64-simulator/Headers/AcurastSDK/