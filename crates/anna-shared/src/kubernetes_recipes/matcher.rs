//! Kubernetes query matching (v0.0.459).

use super::recipes::builtin_recipes;
use super::types::{K8sFeature, K8sRecipe};

/// Detect if a query is about Kubernetes
pub fn detect_feature(query: &str) -> Option<K8sFeature> {
    let lower = query.to_lowercase();

    // First check if it's even a k8s query
    if !is_kubernetes_query(&lower) {
        return None;
    }

    // Find all matching keywords and return the feature with the longest match
    // This prevents "create" from matching before "create namespace"
    let mut best_match: Option<(K8sFeature, usize)> = None;

    for feature in all_features() {
        for keyword in feature.keywords() {
            if lower.contains(keyword) {
                let keyword_len = keyword.len();
                if best_match.is_none() || keyword_len > best_match.unwrap().1 {
                    best_match = Some((feature, keyword_len));
                }
            }
        }
    }

    best_match.map(|(f, _)| f)
}

/// Match a query to a recipe
pub fn match_query(query: &str) -> Option<K8sRecipe> {
    let feature = detect_feature(query)?;

    builtin_recipes()
        .into_iter()
        .find(|r| r.feature == feature)
}

/// Check if query is about Kubernetes
fn is_kubernetes_query(query: &str) -> bool {
    let k8s_indicators = [
        "kubernetes",
        "kubectl",
        "k8s",
        "pod",
        "deployment",
        "service",
        "namespace",
        "cluster",
        "node",
        "replica",
        "container",
        "helm",
        "ingress",
        "configmap",
        "secret",
    ];

    k8s_indicators.iter().any(|k| query.contains(k))
}

/// Get all K8s features
fn all_features() -> Vec<K8sFeature> {
    vec![
        K8sFeature::ListPods,
        K8sFeature::DescribePod,
        K8sFeature::PodLogs,
        K8sFeature::ExecPod,
        K8sFeature::ListDeployments,
        K8sFeature::ScaleDeployment,
        K8sFeature::RestartDeployment,
        K8sFeature::ApplyManifest,
        K8sFeature::DeleteResource,
        K8sFeature::ListServices,
        K8sFeature::PortForward,
        K8sFeature::GetEvents,
        K8sFeature::DebugPod,
        K8sFeature::ClusterHealth,
        K8sFeature::ListNodes,
        K8sFeature::ResourceUsage,
        K8sFeature::ListNamespaces,
        K8sFeature::CreateNamespace,
        K8sFeature::GetConfig,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_pods() {
        assert_eq!(
            detect_feature("list all pods in kubernetes"),
            Some(K8sFeature::ListPods)
        );
    }

    #[test]
    fn test_detect_logs() {
        assert_eq!(
            detect_feature("show me k8s pod logs"),
            Some(K8sFeature::PodLogs)
        );
    }

    #[test]
    fn test_detect_scale() {
        assert_eq!(
            detect_feature("scale deployment to 3 replicas"),
            Some(K8sFeature::ScaleDeployment)
        );
    }

    #[test]
    fn test_not_kubernetes() {
        assert_eq!(detect_feature("how much disk space"), None);
    }

    #[test]
    fn test_match_query() {
        let recipe = match_query("list all kubernetes pods");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().feature, K8sFeature::ListPods);
    }
}
