//! Delta-based greeting insights (system changes).

use crate::greeting_insights::types::GreetingInsight;
use crate::roster::{person_for, Tier};
use crate::snapshot::DeltaItem;
use crate::teams::Team;

/// Add delta-based insights to the collection
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
