//! Service-related greeting insights.

use crate::greeting_insights::types::GreetingInsight;
use crate::roster::{person_for, Tier};
use crate::snapshot::SystemSnapshot;
use crate::teams::Team;

/// Add service-related insights to the collection
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
