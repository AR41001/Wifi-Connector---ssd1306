OLED Display and Wi-Fi Connection Tool with Password Storage

This code provides a user interface for an OLED display to manage Wi-Fi connections on a Raspberry Pi or similar Linux device. It allows users to:

    View available Wi-Fi SSIDs
    Select a Wi-Fi network
    Enter a password for the selected network (using a character selection interface)
    Connect to the Wi-Fi network
    Store Wi-Fi credentials (SSID and password) in a file

Features

    User-friendly menu system for navigation using buttons.
    Character selection interface for entering Wi-Fi passwords securely.
    Saves Wi-Fi credentials (SSID and password) to a file for future use.

Hardware Requirements

    Raspberry Pi or compatible Linux device
    OLED display with I2C interface
    Buttons for navigation (scroll up, scroll down, select, back)
    Reset button (optional)

Software Dependencies

    embedded-graphics - for graphics library functions
    linux-embedded-hal - for GPIO access
    nanomsg - for communication protocols (not used in current implementation)
    rppal - for GPIO control
    ssd1306 - for OLED display driver
    std - Rust standard library

Installation

    Ensure you have Rust installed on your device.
    Copy the code into a project directory.
    Run the following command to build and run the program:

Bash
    
    cargo build
    cargo run

Use code with caution.
Usage

    Power on your device and the OLED display.

    Use the buttons to navigate through the menus:
        Scroll up/down: Navigate through the list of available Wi-Fi networks.
        Select: Choose a Wi-Fi network to connect to.
        Enter password: Select characters to build the password using the scroll buttons and confirm with the select button. Double-click select to move on after entering the password.
        Back: Go back to the previous menu.
        Reset: (Optional) Reset the current operation (might require additional implementation).

    After entering the password and selecting a network, the program will attempt to connect to the Wi-Fi network.

    If successful, it will display a confirmation message and potentially store the credentials in a file (depending on implementation).

Additional Notes

    This is a basic example and may require modifications for specific hardware configurations.
    The code currently uses a placeholder function for saving Wi-Fi credentials to a file. You might need to implement the logic to write the SSID and password to a desired location.
    Error handling and security improvements can be further implemented.

This readme provides a starting point for understanding the code's functionality. Feel free to explore and customize it based on your specific needs.
