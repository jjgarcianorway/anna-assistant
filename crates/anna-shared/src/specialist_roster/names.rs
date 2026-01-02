//! Specialist names and name selection

use crate::specialist_roster::types::{Department, SpecialistLevel};

/// Diverse human names for specialists
pub const SPECIALIST_NAMES: &[(&str, &str)] = &[
    ("Maya", "Desktop"),
    ("Kenji", "Network"),
    ("Fatima", "Security"),
    ("Carlos", "Database"),
    ("Aisha", "DevOps"),
    ("Dmitri", "Sound"),
    ("Priya", "Video"),
    ("Marcus", "Storage"),
    ("Yuki", "Performance"),
    ("Elena", "General"),
    ("Kwame", "Desktop"),
    ("Sofia", "Network"),
    ("Hassan", "Security"),
    ("Mei", "Database"),
    ("Olga", "DevOps"),
    ("Samuel", "Sound"),
    ("Amara", "Video"),
    ("Jin", "Storage"),
    ("Lucia", "Performance"),
    ("Raj", "General"),
];

/// Get a name for a department and level
pub fn get_specialist_name(dept: Department, level: SpecialistLevel) -> &'static str {
    let dept_name = dept.name();
    let idx = match level {
        SpecialistLevel::Junior => 0,
        SpecialistLevel::Senior => 10,
        SpecialistLevel::Lead => 5,
    };

    for (i, (name, d)) in SPECIALIST_NAMES.iter().enumerate() {
        if *d == dept_name && i >= idx {
            return name;
        }
    }

    SPECIALIST_NAMES[0].0
}
