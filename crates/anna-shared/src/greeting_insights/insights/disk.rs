//! Disk-related greeting insights.

use crate::greeting_insights::types::GreetingInsight;
use crate::roster::{person_for, Tier};
use crate::snapshot::SystemSnapshot;
use crate::teams::Team;

/// Add disk-related insights to the collection
pub fn add_disk_insights(snapshot: &SystemSnapshot, insights: &mut Vec<GreetingInsight>) {
    for (mount, percent) in &snapshot.disk {
        if *percent >= 95 {
            let person = person_for(Team::Storage, Tier::Senior);
            insights.push(GreetingInsight {
                staff_name: person.display_name,
                team: Team::Storage,
                message: format!("{} is at {}% - pool status is concerning", mount, percent),
                priority: 95,
                positive: false,
            });
        } else if *percent >= 85 {
            let person = person_for(Team::Storage, Tier::Junior);
            insights.push(GreetingInsight {
                staff_name: person.display_name,
                team: Team::Storage,
                message: format!("{} at {}%, keeping an eye on it", mount, percent),
                priority: 70,
                positive: false,
            });
        }
    }

    // Check for good news - lots of free space
    let all_low = snapshot.disk.values().all(|p| *p < 50);
    if all_low && !snapshot.disk.is_empty() {
        let person = person_for(Team::Storage, Tier::Junior);
        insights.push(GreetingInsight {
            staff_name: person.display_name,
            team: Team::Storage,
            message: "plenty of space on all drives!".to_string(),
            priority: 20,
            positive: true,
        });
    }
}
