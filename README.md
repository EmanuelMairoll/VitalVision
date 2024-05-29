# VitalVision

## Introduction
VitalVision is a cross-platform application designed to visualize real-time data from wearable sensors. Developed as a bachelor project at ETH Zurich's Sensing, Interaction & Perception Lab (SIPLAB), this application connects to multiple wearable devices, streams their sensor data, and provides live visualizations upon the user's request. The app supports sensors similar to those found in devices like the Apple Watch or Fitbit, and it includes features for real-time signal validation and error detection.

## Overall Repository Structure
The VitalVision project is divided into three main components:
1. **Core Library** (`/core`): Handles the core logic of the application, including BLE communication, data storage, and analysis. Written in Rust for performance and safety.
2. **iOS Frontend** (`/ios`): Provides the user interface for iOS devices. Developed in Swift, UI components use SwiftUI and SwiftCharts.
3. **Signal Playgrounds** (`/signal`): Contains Python code used for developing and testing the signal processing algorithms before they were implemented in Rust.

## Build Instructions

### Core
1. Navigate to the `/core` directory.
2. Run the build script:
   ```
   ./build-ios.sh
   ```
   This script runs `cargo build`, generates Swift bindings, and then drops a `.xcframework` into the `ios/Framework` folder.
3. Open the `ios/VitalVision.xcodeproj` file in Xcode.
4. Select your target device and click the play button to build and run the app.

For developing without devices, the app also includes a "Mock Devices" toggle in the settings.

## App Components

### Core
- **lib.rs**: The main entry point of the core library, handling the initialization and coordination of various components. Defines the `VVCore` interface and integrates with UniFFI.
- **BLE Component**: Manages Bluetooth communication, device discovery, and data streaming. Uses the `btleplug` library for BLE operations.
- **Storage Component**: Uses a ring buffer for efficient storage and retrieval of new data points.
- **Analysis Component**: Processes ECG and PPG data to provide real-time quality assessment.

### UniFFI Translation Layer
- **vvcore.udl**: Defines the UniFFI interface, from which Swift, Kotlin and Python bindings can be generated. Allows seamless communication between the Rust core and the Swift frontend.

### iOS Frontend
- **VitalVisionCore**: Handles communication with the Rust core and provides data to the UI. Manages the lifecycle of the Rust core and interfaces with UniFFI.
- **NotificationService**: Manages notifications for low signal quality, ensuring users are alerted when data quality drops.
- **Views**: Various SwiftUI views to visualize devices and their channels, introspect data, see signal quality warnings, watch, and annotate devices. Includes:
  - Device list and detail views
  - Channel data visualization
  - Signal quality indicators
  - Settings and configuration screens
- **AppConfig**: Allows customization of all parameters through user settings, stored using SwiftUI's `@AppStorage`.

## Open TODOs
1. Implement a power-saving "pause" feature, disconnecting all clients temporarily.
2. Author detailed documentation for the different core components and the iOS frontend.
3. Introduce structured logging in the core, removing the stray print statements.
4. Clean up the repo commit history, the recent commits are bit of a mess.
5. Ensure feature parity between the iOS and macOS frontends.

## Acknowledgements
I would like to express my gratitude towards the Sensing, Interaction & Perception Lab at ETH Zurich for their support throughout this project. Special thanks to Manuel Meier for the opportunity and guidance that made this project possible.
