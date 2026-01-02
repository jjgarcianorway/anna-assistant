//! System resource insights (disk, memory, services, deltas).

use crate::roster::{person_for, Tier};
use crate::snapshot::{DeltaItem, SystemSnapshot};
use crate::teams::Team;

use super::types::GreetingInsight;

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

pub fn add_memory_insights(snapshot: &SystemSnapshot, insights: &mut Vec<GreetingInsight>) {
    if snapshot.memory_total_bytes == 0 {
        return;
    }

    let used_percent =
        (snapshot.memory_used_bytes as f64 / snapshot.memory_total_bytes as f64 * 100.0) as u8;

    if used_percent >= 90 {
        let person = person_for(Team::Performance, Tier::Senior);
        insights.push(GreetingInsight {
            staff_name: person.display_name,
            team: Team::Performance,
            message: format!(
                "memory at {}% - that's 90% efficiency... wait",
                used_percent
            ),
            priority: 90,
            positive: false,
        });
    } else if used_percent >= 80 {
        let person = person_for(Team::Performance, Tier::Junior);
        insights.push(GreetingInsight {
            staff_name: person.display_name,
            team: Team::Performance,
            message: format!("*glances at htop* memory at {}%", used_percent),
            priority: 60,
            positive: false,
        });
    } else if used_percent < 30 {
        let person = person_for(Team::Performance, Tier::Junior);
        insights.push(GreetingInsight {
            staff_name: person.display_name,
            team: Team::Performance,
            message: "numbers look good, no bottlenecks!".to_string(),
            priority: 15,
            positive: true,
        });
    }
}

pub fn add_service_insights(snapshot: &SystemSnapshot, insights: &mut Vec<GreetingInsight>) {
    let count = snapshot.failed_services.len();
    if count > 0 {
        let person = person_for(Team::Services, Tier::Senior);
        let msg = if count == 1 {
            format!("{} is in failed state", snapshot.failed_services[0])
        } else {
            format!("{} services in failed state", count)
        };
        insights.push(GreetingInsight {
            staff_name: person.display_name,
            team: Team::Services,
            message: msg,
            priority: 85,
            positive: false,
        });
    }
}

pub fn add_delta_insights(deltas: &[DeltaItem], insights: &mut Vec<GreetingInsight>) {
    for delta in deltas {
        match delta {
            DeltaItem::ServiceRecovered { unit } => {
                let person = person_for(Team::Services, Tier::Senior);
                insights.push(GreetingInsight {
                    staff_name: person.display_name,
                    team: Team::Services,
                    message: format!("{} is back up!", unit),
                    priority: 50,
                    positive: true,
                });
            }
            DeltaItem::DiskCritical { mount, curr, .. } => {
                let person = person_for(Team::Storage, Tier::Senior);
                insights.push(GreetingInsight {
                    staff_name: person.display_name,
                    team: Team::Storage,
                    message: format!("{} just hit {}%!", mount, curr),
                    priority: 95,
                    positive: false,
                });
            }
            DeltaItem::NewFailedService { unit } => {
                let person = person_for(Team::Services, Tier::Junior);
                insights.push(GreetingInsight {
                    staff_name: person.display_name,
                    team: Team::Services,
                    message: format!("{} just failed!", unit),
                    priority: 85,
                    positive: false,
                });
            }
            _ => {} // Other deltas handled elsewhere
        }
    }
}
