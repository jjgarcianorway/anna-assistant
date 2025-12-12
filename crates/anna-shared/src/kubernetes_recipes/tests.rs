//! Tests for Kubernetes recipes (v0.0.459).

use super::*;

#[test]
fn test_detect_list_pods() {
    assert_eq!(
        detect_feature("list all pods in kubernetes"),
        Some(K8sFeature::ListPods)
    );
    assert_eq!(
        detect_feature("show me k8s pods"),
        Some(K8sFeature::ListPods)
    );
    assert_eq!(
        detect_feature("kubectl get pods"),
        Some(K8sFeature::ListPods)
    );
}

#[test]
fn test_detect_pod_logs() {
    assert_eq!(
        detect_feature("show pod logs"),
        Some(K8sFeature::PodLogs)
    );
    assert_eq!(
        detect_feature("view container logs in kubernetes"),
        Some(K8sFeature::PodLogs)
    );
}

#[test]
fn test_detect_describe_pod() {
    assert_eq!(
        detect_feature("describe kubernetes pod"),
        Some(K8sFeature::DescribePod)
    );
    assert_eq!(
        detect_feature("get pod details in k8s"),
        Some(K8sFeature::DescribePod)
    );
}

#[test]
fn test_detect_exec_pod() {
    assert_eq!(
        detect_feature("exec into kubernetes pod"),
        Some(K8sFeature::ExecPod)
    );
    assert_eq!(
        detect_feature("get shell in k8s container"),
        Some(K8sFeature::ExecPod)
    );
    assert_eq!(
        detect_feature("run bash in pod"),
        Some(K8sFeature::ExecPod)
    );
}

#[test]
fn test_detect_deployments() {
    assert_eq!(
        detect_feature("list kubernetes deployments"),
        Some(K8sFeature::ListDeployments)
    );
    assert_eq!(
        detect_feature("get deployments in k8s"),
        Some(K8sFeature::ListDeployments)
    );
}

#[test]
fn test_detect_scale() {
    // "scale up" and "scale down" are specific enough
    assert_eq!(
        detect_feature("scale up kubernetes app"),
        Some(K8sFeature::ScaleDeployment)
    );
    assert_eq!(
        detect_feature("scale down k8s"),
        Some(K8sFeature::ScaleDeployment)
    );
}

#[test]
fn test_detect_restart() {
    assert_eq!(
        detect_feature("rollout restart kubernetes"),
        Some(K8sFeature::RestartDeployment)
    );
    assert_eq!(
        detect_feature("redeploy k8s app"),
        Some(K8sFeature::RestartDeployment)
    );
}

#[test]
fn test_detect_apply() {
    assert_eq!(
        detect_feature("apply kubernetes manifest"),
        Some(K8sFeature::ApplyManifest)
    );
    assert_eq!(
        detect_feature("deploy yaml to k8s"),
        Some(K8sFeature::ApplyManifest)
    );
}

#[test]
fn test_detect_delete() {
    assert_eq!(
        detect_feature("delete kubernetes resource"),
        Some(K8sFeature::DeleteResource)
    );
    assert_eq!(
        detect_feature("remove pod from k8s"),
        Some(K8sFeature::DeleteResource)
    );
}

#[test]
fn test_detect_services() {
    assert_eq!(
        detect_feature("list kubernetes services"),
        Some(K8sFeature::ListServices)
    );
    assert_eq!(
        detect_feature("get svc in k8s"),
        Some(K8sFeature::ListServices)
    );
}

#[test]
fn test_detect_port_forward() {
    assert_eq!(
        detect_feature("port-forward kubernetes pod"),
        Some(K8sFeature::PortForward)
    );
    assert_eq!(
        detect_feature("tunnel to k8s service"),
        Some(K8sFeature::PortForward)
    );
}

#[test]
fn test_detect_events() {
    assert_eq!(
        detect_feature("get kubernetes events"),
        Some(K8sFeature::GetEvents)
    );
    assert_eq!(
        detect_feature("what happened in k8s cluster"),
        Some(K8sFeature::GetEvents)
    );
}

#[test]
fn test_detect_debug() {
    assert_eq!(
        detect_feature("debug kubernetes pod"),
        Some(K8sFeature::DebugPod)
    );
    assert_eq!(
        detect_feature("k8s pod not working"),
        Some(K8sFeature::DebugPod)
    );
    assert_eq!(
        detect_feature("pod crash in kubernetes"),
        Some(K8sFeature::DebugPod)
    );
}

#[test]
fn test_detect_cluster_health() {
    assert_eq!(
        detect_feature("kubernetes cluster health"),
        Some(K8sFeature::ClusterHealth)
    );
    assert_eq!(
        detect_feature("check k8s cluster status"),
        Some(K8sFeature::ClusterHealth)
    );
}

#[test]
fn test_detect_nodes() {
    assert_eq!(
        detect_feature("list kubernetes nodes"),
        Some(K8sFeature::ListNodes)
    );
    assert_eq!(
        detect_feature("get cluster nodes"),
        Some(K8sFeature::ListNodes)
    );
}

#[test]
fn test_detect_resource_usage() {
    // Use "top" keyword which is specific to ResourceUsage
    assert_eq!(
        detect_feature("kubectl top in kubernetes"),
        Some(K8sFeature::ResourceUsage)
    );
    assert_eq!(
        detect_feature("k8s resources consumption"),
        Some(K8sFeature::ResourceUsage)
    );
}

#[test]
fn test_detect_namespaces() {
    assert_eq!(
        detect_feature("list kubernetes namespaces"),
        Some(K8sFeature::ListNamespaces)
    );
    assert_eq!(
        detect_feature("get ns in k8s"),
        Some(K8sFeature::ListNamespaces)
    );
}

#[test]
fn test_detect_create_namespace() {
    // Note: "create namespace" is more specific than just "create"
    assert_eq!(
        detect_feature("create namespace in k8s"),
        Some(K8sFeature::CreateNamespace)
    );
    assert_eq!(
        detect_feature("new namespace kubernetes"),
        Some(K8sFeature::CreateNamespace)
    );
}

#[test]
fn test_detect_config() {
    assert_eq!(
        detect_feature("get kubernetes configmap"),
        Some(K8sFeature::GetConfig)
    );
    assert_eq!(
        detect_feature("view k8s secrets"),
        Some(K8sFeature::GetConfig)
    );
}

#[test]
fn test_not_kubernetes_query() {
    assert_eq!(detect_feature("how much disk space"), None);
    assert_eq!(detect_feature("install htop"), None);
    assert_eq!(detect_feature("restart docker"), None);
}

#[test]
fn test_match_query_returns_recipe() {
    let recipe = match_query("list all kubernetes pods");
    assert!(recipe.is_some());
    let recipe = recipe.unwrap();
    assert_eq!(recipe.feature, K8sFeature::ListPods);
    assert!(!recipe.commands.is_empty());
    assert!(!recipe.answer_template.is_empty());
}

#[test]
fn test_all_features_have_recipes() {
    let recipes = builtin_recipes();
    let features: Vec<K8sFeature> = recipes.iter().map(|r| r.feature).collect();

    // Check all features have corresponding recipes
    assert!(features.contains(&K8sFeature::ListPods));
    assert!(features.contains(&K8sFeature::DescribePod));
    assert!(features.contains(&K8sFeature::PodLogs));
    assert!(features.contains(&K8sFeature::ExecPod));
    assert!(features.contains(&K8sFeature::ListDeployments));
    assert!(features.contains(&K8sFeature::ScaleDeployment));
    assert!(features.contains(&K8sFeature::RestartDeployment));
    assert!(features.contains(&K8sFeature::ApplyManifest));
    assert!(features.contains(&K8sFeature::DeleteResource));
    assert!(features.contains(&K8sFeature::ListServices));
    assert!(features.contains(&K8sFeature::PortForward));
    assert!(features.contains(&K8sFeature::GetEvents));
    assert!(features.contains(&K8sFeature::DebugPod));
    assert!(features.contains(&K8sFeature::ClusterHealth));
    assert!(features.contains(&K8sFeature::ListNodes));
    assert!(features.contains(&K8sFeature::ResourceUsage));
    assert!(features.contains(&K8sFeature::ListNamespaces));
    assert!(features.contains(&K8sFeature::CreateNamespace));
    assert!(features.contains(&K8sFeature::GetConfig));
}

#[test]
fn test_recipes_have_required_fields() {
    for recipe in builtin_recipes() {
        assert!(
            !recipe.commands.is_empty(),
            "Recipe {:?} has no commands",
            recipe.feature
        );
        assert!(
            !recipe.answer_template.is_empty(),
            "Recipe {:?} has no answer template",
            recipe.feature
        );
        assert!(
            !recipe.description.is_empty(),
            "Recipe {:?} has no description",
            recipe.feature
        );
        assert!(
            recipe.requires.contains(&"kubectl".to_string()),
            "Recipe {:?} should require kubectl",
            recipe.feature
        );
    }
}

#[test]
fn test_feature_display_names() {
    assert_eq!(K8sFeature::ListPods.display_name(), "list pods");
    assert_eq!(K8sFeature::PodLogs.display_name(), "view pod logs");
    assert_eq!(K8sFeature::ScaleDeployment.display_name(), "scale deployment");
    assert_eq!(K8sFeature::DebugPod.display_name(), "debug pod");
}

#[test]
fn test_feature_keywords() {
    let keywords = K8sFeature::ListPods.keywords();
    assert!(keywords.contains(&"pods"));
    assert!(keywords.contains(&"list pods"));

    let keywords = K8sFeature::ScaleDeployment.keywords();
    assert!(keywords.contains(&"scale"));
    assert!(keywords.contains(&"replicas"));
}

#[test]
fn test_recipe_builder() {
    let recipe = K8sRecipe::new(K8sFeature::ListPods, "Test description")
        .with_command("kubectl get pods")
        .with_answer("Test answer")
        .with_note("Test note")
        .with_requires("helm");

    assert_eq!(recipe.feature, K8sFeature::ListPods);
    assert_eq!(recipe.description, "Test description");
    assert_eq!(recipe.commands, vec!["kubectl get pods"]);
    assert_eq!(recipe.answer_template, "Test answer");
    assert_eq!(recipe.notes, vec!["Test note"]);
    assert!(recipe.requires.contains(&"kubectl".to_string()));
    assert!(recipe.requires.contains(&"helm".to_string()));
}

#[test]
fn test_manifest_example() {
    let recipes = builtin_recipes();
    let apply_recipe = recipes
        .iter()
        .find(|r| r.feature == K8sFeature::ApplyManifest)
        .unwrap();

    assert!(apply_recipe.manifest_example.is_some());
    let manifest = apply_recipe.manifest_example.as_ref().unwrap();
    assert!(manifest.contains("apiVersion"));
    assert!(manifest.contains("kind: Deployment"));
}
