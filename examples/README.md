# Greenhouse Backend Examples

This directory contains example implementations demonstrating how to use the Greenhouse Backend framework to create smart devices and services.

## Overview

The examples showcase practical implementations of the Greenhouse Backend's smart device interface, providing working code that you can use as a reference for building your own devices and services.

## Examples

### Input/Output Integer Saver (`input_output_int_saver.rs`)

A complete example of a hybrid smart device that can both read and write integer values with validation.

#### Features

- **Hybrid Device**: Implements both input and output functionality
- **Configuration Management**: Uses configurable min/max validation bounds
- **REST API**: Exposes HTTP endpoints for device interaction
- **Status Reporting**: Provides device status information
- **Configurable Port**: Can be configured to run on different ports

#### How it Works

The device maintains an in-memory integer value that can be:
- **Read**: Returns the current saved value
- **Written**: Updates the value with validation against min/max bounds
- **Configured**: Allows runtime configuration of validation parameters

#### Configuration

The device uses a JSON configuration file with the following structure:

```json
{
  "mode": "InputOutput",
  "port": 6001,
  "input_type": "Number",
  "output_type": "Number",
  "additional_config": {
    "min": 0,
    "max": 100
  }
}
```

#### API Endpoints

- `GET /read` - Returns the current saved integer value
- `POST /write` - Updates the saved value (with validation)
- `GET /status` - Returns device status information
- `POST /config` - Updates device configuration

#### Running the Example

**Local Development:**
```bash
# Build and run with default config
cargo run --example input_output_int_saver

# Run with custom config path
cargo run --example input_output_int_saver /path/to/config.json
```

## Project Structure

```
examples/
├── Cargo.toml              # Dependencies and build 
├── input_output_int_saver.rs # Main example implementation
└── README.md               # This file
```

## Running

**Run a specific example:**
   ```bash
   cargo run --example input_output_int_saver
   ```

## Extending the Examples

To create your own device based on these examples:

1. **Copy the structure** from `input_output_int_saver.rs`
2. **Implement your handlers** for read, write, status, and config operations
3. **Define your configuration** structure
4. **Add your example** to the `Cargo.toml` file

## Troubleshooting

### Common Issues

- **Port already in use**: Change the port in the configuration file
- **Permission denied**: Ensure the config directory is writable
