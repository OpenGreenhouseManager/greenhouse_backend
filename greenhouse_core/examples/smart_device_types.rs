//! Demonstrates the [`Type`] enum used for sensor readings and actuator commands.
//!
//! The `Type` enum is the core data type exchanged between the greenhouse backend
//! and smart devices. This example shows all variants and a round-trip
//! serialisation test.
//!
//! Run with:
//! ```bash
//! cargo run --example smart_device_types --features smart_device_dto
//! ```

use greenhouse_core::smart_device_dto::{Measurement, Type};

fn main() {
    let examples: Vec<(&str, Type)> = vec![
        ("Number", Type::Number(22.5)),
        ("Boolean (on)", Type::Boolean(true)),
        (
            "Measurement (°C)",
            Type::Measurement(Measurement {
                value: 23.4,
                unit: "°C".to_string(),
            }),
        ),
        (
            "Measurement (%RH)",
            Type::Measurement(Measurement {
                value: 65.0,
                unit: "%RH".to_string(),
            }),
        ),
        ("Stream", Type::Stream),
        ("None", Type::None),
    ];

    for (name, value) in &examples {
        let json = serde_json::to_string(value).unwrap();
        println!("{name:20} → {json}");
    }

    // Round-trip: serialise then deserialise
    let original = Type::Measurement(Measurement {
        value: 101.325,
        unit: "kPa".to_string(),
    });
    let json = serde_json::to_string(&original).unwrap();
    let decoded: Type = serde_json::from_str(&json).unwrap();

    match decoded {
        Type::Measurement(m) => {
            assert!((m.value - 101.325).abs() < f64::EPSILON);
            assert_eq!(m.unit, "kPa");
            println!("\nRound-trip OK: {} {}", m.value, m.unit);
        }
        _ => panic!("unexpected type after round-trip"),
    }
}
