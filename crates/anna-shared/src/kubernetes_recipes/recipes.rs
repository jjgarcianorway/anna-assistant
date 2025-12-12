//! Kubernetes builtin recipes (v0.0.459).

use super::types::{K8sFeature, K8sRecipe};

/// Get all builtin Kubernetes recipes
pub fn builtin_recipes() -> Vec<K8sRecipe> {
    vec![
        // Pod operations
        K8sRecipe::new(K8sFeature::ListPods, "List all pods in a namespace")
            .with_command("kubectl get pods")
            .with_command("kubectl get pods -n <namespace>")
            .with_command("kubectl get pods --all-namespaces")
            .with_command("kubectl get pods -o wide")
            .with_answer(
                "To list pods, use `kubectl get pods`. Add `-n <namespace>` for a specific \
                 namespace, or `--all-namespaces` for all. Use `-o wide` for more details.",
            )
            .with_note("Use -o yaml or -o json for full resource details"),
        K8sRecipe::new(K8sFeature::DescribePod, "Get detailed pod information")
            .with_command("kubectl describe pod <pod-name>")
            .with_command("kubectl describe pod <pod-name> -n <namespace>")
            .with_answer(
                "Use `kubectl describe pod <name>` to see detailed info including events, \
                 conditions, and container status.",
            )
            .with_note("Events section shows recent pod lifecycle changes"),
        K8sRecipe::new(K8sFeature::PodLogs, "View pod logs")
            .with_command("kubectl logs <pod-name>")
            .with_command("kubectl logs <pod-name> -c <container>")
            .with_command("kubectl logs <pod-name> --previous")
            .with_command("kubectl logs -f <pod-name>")
            .with_command("kubectl logs --tail=100 <pod-name>")
            .with_answer(
                "Use `kubectl logs <pod>` to view logs. Add `-f` to follow, `--previous` for \
                 crashed container logs, `-c <container>` for multi-container pods.",
            )
            .with_note("--previous shows logs from previous container instance (useful for crashes)"),
        K8sRecipe::new(K8sFeature::ExecPod, "Execute command in pod")
            .with_command("kubectl exec -it <pod-name> -- /bin/bash")
            .with_command("kubectl exec -it <pod-name> -- /bin/sh")
            .with_command("kubectl exec <pod-name> -- <command>")
            .with_answer(
                "Use `kubectl exec -it <pod> -- /bin/bash` to get a shell. For non-interactive \
                 commands: `kubectl exec <pod> -- <command>`.",
            )
            .with_note("Use /bin/sh if bash is not available in the container"),
        // Deployment operations
        K8sRecipe::new(K8sFeature::ListDeployments, "List deployments")
            .with_command("kubectl get deployments")
            .with_command("kubectl get deployments -n <namespace>")
            .with_command("kubectl get deploy -o wide")
            .with_answer(
                "Use `kubectl get deployments` to list all deployments. Add `-o wide` for \
                 more details including container images.",
            ),
        K8sRecipe::new(K8sFeature::ScaleDeployment, "Scale deployment replicas")
            .with_command("kubectl scale deployment <name> --replicas=<count>")
            .with_command("kubectl scale --replicas=3 deployment/<name>")
            .with_answer(
                "Use `kubectl scale deployment <name> --replicas=<n>` to change replica count. \
                 The cluster will automatically create or terminate pods.",
            )
            .with_note("HPA (Horizontal Pod Autoscaler) may override manual scaling"),
        K8sRecipe::new(K8sFeature::RestartDeployment, "Restart deployment pods")
            .with_command("kubectl rollout restart deployment <name>")
            .with_command("kubectl rollout status deployment <name>")
            .with_command("kubectl rollout undo deployment <name>")
            .with_answer(
                "Use `kubectl rollout restart deployment <name>` to restart all pods. \
                 Use `rollout status` to monitor, `rollout undo` to rollback.",
            )
            .with_note("This creates new pods and terminates old ones gracefully"),
        // Resource management
        K8sRecipe::new(K8sFeature::ApplyManifest, "Apply Kubernetes manifest")
            .with_command("kubectl apply -f <file.yaml>")
            .with_command("kubectl apply -f <directory>/")
            .with_command("kubectl apply -k <kustomize-dir>/")
            .with_manifest(
                r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-app
spec:
  replicas: 3
  selector:
    matchLabels:
      app: my-app
  template:
    metadata:
      labels:
        app: my-app
    spec:
      containers:
      - name: my-app
        image: my-image:latest
        ports:
        - containerPort: 8080"#,
            )
            .with_answer(
                "Use `kubectl apply -f <file.yaml>` to create or update resources. \
                 This is declarative - the same command works for create and update.",
            )
            .with_note("Use -k for Kustomize directories"),
        K8sRecipe::new(K8sFeature::DeleteResource, "Delete Kubernetes resource")
            .with_command("kubectl delete <resource> <name>")
            .with_command("kubectl delete -f <file.yaml>")
            .with_command("kubectl delete pod <name> --grace-period=0 --force")
            .with_answer(
                "Use `kubectl delete <resource> <name>` to remove resources. \
                 Use `--force --grace-period=0` for stuck pods (use carefully!).",
            )
            .with_note("Deleting a deployment will also delete its pods"),
        // Services and networking
        K8sRecipe::new(K8sFeature::ListServices, "List services")
            .with_command("kubectl get services")
            .with_command("kubectl get svc -o wide")
            .with_command("kubectl describe service <name>")
            .with_answer(
                "Use `kubectl get services` (or `svc`) to list services. \
                 The EXTERNAL-IP column shows load balancer IPs.",
            ),
        K8sRecipe::new(K8sFeature::PortForward, "Port forward to pod or service")
            .with_command("kubectl port-forward pod/<name> <local>:<remote>")
            .with_command("kubectl port-forward svc/<name> <local>:<remote>")
            .with_command("kubectl port-forward deployment/<name> 8080:80")
            .with_answer(
                "Use `kubectl port-forward <resource> <local>:<remote>` to access pods \
                 locally. Example: `kubectl port-forward pod/myapp 8080:80`.",
            )
            .with_note("Press Ctrl+C to stop the tunnel"),
        // Debugging
        K8sRecipe::new(K8sFeature::GetEvents, "Get cluster events")
            .with_command("kubectl get events")
            .with_command("kubectl get events --sort-by='.lastTimestamp'")
            .with_command("kubectl get events -n <namespace>")
            .with_answer(
                "Use `kubectl get events` to see cluster events. Add `--sort-by='.lastTimestamp'` \
                 to see most recent first.",
            )
            .with_note("Events show pod scheduling, pulling images, and errors"),
        K8sRecipe::new(K8sFeature::DebugPod, "Debug pod issues")
            .with_command("kubectl describe pod <name>")
            .with_command("kubectl logs <name> --previous")
            .with_command("kubectl get events --field-selector involvedObject.name=<pod>")
            .with_command("kubectl debug -it <pod> --image=busybox")
            .with_answer(
                "To debug a pod: 1) `describe` to check events and status, \
                 2) `logs --previous` for crash logs, 3) `get events` for scheduling issues, \
                 4) `debug` to attach a debug container.",
            )
            .with_note("Common issues: ImagePullBackOff, CrashLoopBackOff, Pending"),
        // Cluster info
        K8sRecipe::new(K8sFeature::ClusterHealth, "Check cluster health")
            .with_command("kubectl cluster-info")
            .with_command("kubectl get componentstatuses")
            .with_command("kubectl get nodes")
            .with_command("kubectl top nodes")
            .with_answer(
                "Check cluster health with `kubectl cluster-info` and `kubectl get nodes`. \
                 Use `kubectl top nodes` to see resource usage (requires metrics-server).",
            )
            .with_note("All nodes should be in Ready state"),
        K8sRecipe::new(K8sFeature::ListNodes, "List cluster nodes")
            .with_command("kubectl get nodes")
            .with_command("kubectl get nodes -o wide")
            .with_command("kubectl describe node <name>")
            .with_command("kubectl top nodes")
            .with_answer(
                "Use `kubectl get nodes` to see cluster nodes. Add `-o wide` for IPs and \
                 versions. Use `describe node` for detailed info.",
            ),
        K8sRecipe::new(K8sFeature::ResourceUsage, "Check resource usage")
            .with_command("kubectl top pods")
            .with_command("kubectl top pods -n <namespace>")
            .with_command("kubectl top nodes")
            .with_answer(
                "Use `kubectl top pods` to see CPU/memory usage. Use `top nodes` for \
                 node-level metrics. Requires metrics-server to be installed.",
            )
            .with_note("metrics-server must be installed in the cluster"),
        // Namespaces
        K8sRecipe::new(K8sFeature::ListNamespaces, "List namespaces")
            .with_command("kubectl get namespaces")
            .with_command("kubectl get ns")
            .with_answer(
                "Use `kubectl get namespaces` (or `ns`) to list all namespaces. \
                 Default namespaces: default, kube-system, kube-public.",
            ),
        K8sRecipe::new(K8sFeature::CreateNamespace, "Create namespace")
            .with_command("kubectl create namespace <name>")
            .with_command("kubectl apply -f namespace.yaml")
            .with_manifest(
                r#"apiVersion: v1
kind: Namespace
metadata:
  name: my-namespace"#,
            )
            .with_answer(
                "Create namespace with `kubectl create namespace <name>` or apply a manifest. \
                 Namespaces isolate resources within the cluster.",
            ),
        // Config
        K8sRecipe::new(K8sFeature::GetConfig, "Get ConfigMaps and Secrets")
            .with_command("kubectl get configmaps")
            .with_command("kubectl get secrets")
            .with_command("kubectl describe configmap <name>")
            .with_command("kubectl get secret <name> -o jsonpath='{.data}'")
            .with_answer(
                "Use `kubectl get configmaps` and `kubectl get secrets` to list configs. \
                 Secret data is base64 encoded - decode with `echo <data> | base64 -d`.",
            )
            .with_note("Secrets are base64 encoded, not encrypted at rest by default"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipes_exist() {
        let recipes = builtin_recipes();
        assert!(!recipes.is_empty());
    }

    #[test]
    fn test_all_features_covered() {
        let recipes = builtin_recipes();
        let features: Vec<_> = recipes.iter().map(|r| r.feature).collect();

        // Check key features are covered
        assert!(features.contains(&K8sFeature::ListPods));
        assert!(features.contains(&K8sFeature::PodLogs));
        assert!(features.contains(&K8sFeature::ScaleDeployment));
        assert!(features.contains(&K8sFeature::DebugPod));
    }

    #[test]
    fn test_recipes_have_commands() {
        for recipe in builtin_recipes() {
            assert!(
                !recipe.commands.is_empty(),
                "Recipe {:?} has no commands",
                recipe.feature
            );
        }
    }

    #[test]
    fn test_recipes_have_answers() {
        for recipe in builtin_recipes() {
            assert!(
                !recipe.answer_template.is_empty(),
                "Recipe {:?} has no answer",
                recipe.feature
            );
        }
    }
}
