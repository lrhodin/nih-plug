//
//  NIHPlugAUv3ViewController.swift
//  NIH-Plug AUv3
//
//  Created by Implementation Ralph.
//  Copyright © 2025 NIH-Plug. All rights reserved.
//

import Cocoa
import CoreAudioKit

/// View controller for NIH-Plug AUv3 plugin UI.
/// This provides the user interface for the Audio Unit extension.
@objc public class NIHPlugAUv3ViewController: AUViewController, AUAudioUnitFactory {

    // MARK: - Properties

    /// The audio unit instance managed by this view controller.
    private var audioUnit: NIHPlugAUv3?

    // MARK: - View Lifecycle

    public override func loadView() {
        print("NIHPlugAUv3ViewController: loadView called")

        // Create a simple view with plugin information
        let containerView = NSView(frame: NSRect(x: 0, y: 0, width: 400, height: 300))
        containerView.wantsLayer = true
        containerView.layer?.backgroundColor = NSColor.windowBackgroundColor.cgColor

        // Create a label with plugin name
        let label = NSTextField(labelWithString: "NIH-Plug AUv3")
        label.font = NSFont.systemFont(ofSize: 24, weight: .bold)
        label.alignment = .center
        label.translatesAutoresizingMaskIntoConstraints = false
        containerView.addSubview(label)

        // Create a subtitle label
        let subtitle = NSTextField(labelWithString: "Audio Unit v3 Plugin")
        subtitle.font = NSFont.systemFont(ofSize: 14)
        subtitle.textColor = .secondaryLabelColor
        subtitle.alignment = .center
        subtitle.translatesAutoresizingMaskIntoConstraints = false
        containerView.addSubview(subtitle)

        // Create a status label
        let statusLabel = NSTextField(labelWithString: "Ready")
        statusLabel.font = NSFont.systemFont(ofSize: 12)
        statusLabel.textColor = .tertiaryLabelColor
        statusLabel.alignment = .center
        statusLabel.translatesAutoresizingMaskIntoConstraints = false
        containerView.addSubview(statusLabel)

        // Layout constraints
        NSLayoutConstraint.activate([
            label.centerXAnchor.constraint(equalTo: containerView.centerXAnchor),
            label.centerYAnchor.constraint(equalTo: containerView.centerYAnchor, constant: -40),

            subtitle.centerXAnchor.constraint(equalTo: containerView.centerXAnchor),
            subtitle.topAnchor.constraint(equalTo: label.bottomAnchor, constant: 8),

            statusLabel.centerXAnchor.constraint(equalTo: containerView.centerXAnchor),
            statusLabel.topAnchor.constraint(equalTo: subtitle.bottomAnchor, constant: 20)
        ])

        self.view = containerView

        print("NIHPlugAUv3ViewController: View loaded successfully")
    }

    public override func viewDidLoad() {
        super.viewDidLoad()
        print("NIHPlugAUv3ViewController: viewDidLoad called")
    }

    // MARK: - AUAudioUnitFactory Protocol

    /// Creates an instance of the audio unit.
    /// This is called by the system to instantiate the Audio Unit.
    @objc public func createAudioUnit(with componentDescription: AudioComponentDescription) throws -> AUAudioUnit {
        print("NIHPlugAUv3ViewController: createAudioUnit called")
        print("NIHPlugAUv3ViewController: Component description - type: \(componentDescription.componentType), subtype: \(componentDescription.componentSubType), manufacturer: \(componentDescription.componentManufacturer)")

        let audioUnit = try NIHPlugAUv3(componentDescription: componentDescription, options: [])
        self.audioUnit = audioUnit

        print("NIHPlugAUv3ViewController: Audio unit created successfully")
        return audioUnit
    }

    // MARK: - Public Methods

    /// Selects the view configuration for the given width.
    /// This is called by the host to determine which view size to use.
    @available(macOS 10.13, *)
    public func selectViewConfiguration(_ viewConfiguration: AUAudioUnitViewConfiguration) {
        print("NIHPlugAUv3ViewController: selectViewConfiguration called - width: \(viewConfiguration.width), height: \(viewConfiguration.height)")
    }
}
