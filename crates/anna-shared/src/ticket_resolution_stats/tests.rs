//! Tests for ticket resolution statistics

#[cfg(test)]
mod tests {
    use crate::ticket_resolution_stats::{
        format_resolution_stats, is_resolution_stats_query, resolution_fun_fact,
        ResolutionMethod, ResolutionRecord, Resolver, TicketResolutionStats,
    };

    fn make_resolution(resolver: Resolver, method: ResolutionMethod) -> ResolutionRecord {
        ResolutionRecord {
            ticket_id: format!("TKT-{:?}", resolver),
            resolver,
            method,
            department: Some("Desktop".to_string()),
            specialist_name: None,
            resolved_at: 1234567890,
            resolution_time_secs: 60,
            recipe_learned: false,
            confidence: Some(85),
        }
    }

    #[test]
    fn test_resolver() {
        assert_eq!(Resolver::Anna.name(), "Anna");
        assert_eq!(Resolver::Junior.symbol(), "J");
        assert!(Resolver::Senior.is_specialist());
        assert!(!Resolver::Anna.is_specialist());
    }

    #[test]
    fn test_resolution_method() {
        assert_eq!(ResolutionMethod::Recipe.name(), "Recipe");
        assert_eq!(ResolutionMethod::Specialist.name(), "Specialist");
    }

    #[test]
    fn test_record_resolution() {
        let mut stats = TicketResolutionStats::new();
        stats.record(make_resolution(Resolver::Anna, ResolutionMethod::Recipe));

        assert_eq!(stats.total_count(), 1);
        assert_eq!(stats.anna_count, 1);
    }

    #[test]
    fn test_anna_rate() {
        let mut stats = TicketResolutionStats::new();
        stats.record(make_resolution(Resolver::Anna, ResolutionMethod::Recipe));
        stats.record(make_resolution(Resolver::Anna, ResolutionMethod::Recipe));
        stats.record(make_resolution(Resolver::Junior, ResolutionMethod::Specialist));

        assert!((stats.anna_rate() - 66.66).abs() < 1.0);
    }

    #[test]
    fn test_by_resolver() {
        let mut stats = TicketResolutionStats::new();
        stats.record(make_resolution(Resolver::Anna, ResolutionMethod::Recipe));
        stats.record(make_resolution(Resolver::Junior, ResolutionMethod::Specialist));

        assert_eq!(stats.by_res(Resolver::Anna).len(), 1);
        assert_eq!(stats.by_res(Resolver::Junior).len(), 1);
    }

    #[test]
    fn test_recipe_learned() {
        let mut stats = TicketResolutionStats::new();
        let mut resolution = make_resolution(Resolver::Anna, ResolutionMethod::Recipe);
        resolution.recipe_learned = true;
        stats.record(resolution);

        assert_eq!(stats.recipes_learned, 1);
    }

    #[test]
    fn test_avg_resolution_time() {
        let mut stats = TicketResolutionStats::new();
        let mut r1 = make_resolution(Resolver::Anna, ResolutionMethod::Recipe);
        r1.resolution_time_secs = 30;
        let mut r2 = make_resolution(Resolver::Anna, ResolutionMethod::Recipe);
        r2.resolution_time_secs = 90;

        stats.record(r1);
        stats.record(r2);

        assert_eq!(stats.avg_resolution_time(), 60.0);
    }

    #[test]
    fn test_format_resolution_stats() {
        let mut stats = TicketResolutionStats::new();
        stats.record(make_resolution(Resolver::Anna, ResolutionMethod::Recipe));

        let output = format_resolution_stats(&stats);
        assert!(output.contains("Resolution Stats"));
        assert!(output.contains("Anna: 1"));
    }

    #[test]
    fn test_is_resolution_stats_query() {
        assert!(is_resolution_stats_query("show resolution stats"));
        assert!(is_resolution_stats_query("how many tickets resolved?"));
        assert!(is_resolution_stats_query("anna vs specialist stats"));
        assert!(!is_resolution_stats_query("what is the weather?"));
    }

    #[test]
    fn test_resolution_fun_fact() {
        let mut stats = TicketResolutionStats::new();
        stats.record(make_resolution(Resolver::Anna, ResolutionMethod::Recipe));

        let fact = resolution_fun_fact(&stats);
        assert!(!fact.is_empty());
    }
}
