package main

import (
    "context"
    "encoding/json"
    "fmt"
    "log"
    "net/http"
    "os"
    "path/filepath"
    "strings"
    "sync"

    corev1 "k8s.io/api/core/v1"
    metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
    "k8s.io/client-go/kubernetes"
    "k8s.io/client-go/rest"
    "k8s.io/client-go/tools/clientcmd"
    "k8s.io/client-go/util/homedir"
)

// NodeIPResponse represents the response structure for node IP information
type NodeIPResponse struct {
    NodeName   string `json:"nodeName"`
    ExternalIP string `json:"externalIP"`
    InternalIP string `json:"internalIP"`
    Error      string `json:"error,omitempty"`
}

// KubeClient wraps the Kubernetes client
type KubeClient struct {
    clientset *kubernetes.Clientset
    config    *rest.Config
    mutex     sync.RWMutex
}

// NewKubeClient creates a new Kubernetes client using the default kubeconfig resolution
func NewKubeClient() (*KubeClient, error) {
    config, err := buildKubeConfig()
    if err != nil {
        return nil, err
    }

    // Create the kubernetes clientset
    clientset, err := kubernetes.NewForConfig(config)
    if err != nil {
        return nil, fmt.Errorf("failed to create kubernetes client: %v", err)
    }

    return &KubeClient{
        clientset: clientset,
        config:    config,
    }, nil
}

// buildKubeConfig builds the kubernetes configuration
func buildKubeConfig() (*rest.Config, error) {
    var config *rest.Config
    var err error

    // Try to use in-cluster config first (for pods running inside cluster)
    config, err = rest.InClusterConfig()
    if err != nil {
        // If in-cluster config fails, try to use kubeconfig file
        var kubeconfig string

        // Check KUBECONFIG environment variable first
        if kubeconfigEnv := os.Getenv("KUBECONFIG"); kubeconfigEnv != "" {
            kubeconfig = kubeconfigEnv
        } else {
            // Fall back to default kubeconfig location
            if home := homedir.HomeDir(); home != "" {
                kubeconfig = filepath.Join(home, ".kube", "config")
            }
        }

        // Build config from kubeconfig file
        config, err = clientcmd.BuildConfigFromFlags("", kubeconfig)
        if err != nil {
            return nil, fmt.Errorf("failed to build kubeconfig: %v", err)
        }
    }

    return config, nil
}

// reconnect attempts to reconnect to the Kubernetes cluster
func (kc *KubeClient) reconnect() error {
    kc.mutex.Lock()
    defer kc.mutex.Unlock()

    log.Println("Attempting to reconnect to Kubernetes cluster...")

    // Create new clientset using the stored config
    clientset, err := kubernetes.NewForConfig(kc.config)
    if err != nil {
        return fmt.Errorf("failed to reconnect to kubernetes client: %v", err)
    }

    kc.clientset = clientset
    log.Println("Successfully reconnected to Kubernetes cluster")
    return nil
}

// isConnectionError checks if the error is related to connection issues
func isConnectionError(err error) bool {
    if err == nil {
        return false
    }

    errMsg := strings.ToLower(err.Error())
    connectionErrors := []string{
        "connection refused",
        "connection reset",
        "connection timeout",
        "no such host",
        "network is unreachable",
        "i/o timeout",
        "context deadline exceeded",
        "server closed idle connection",
        "unable to connect to the server",
        "the server could not find the requested resource",
        "dial tcp",
    }

    for _, connErr := range connectionErrors {
        if strings.Contains(errMsg, connErr) {
            return true
        }
    }

    return false
}

// GetNodeIPs retrieves the external and internal IPs for a given node
func (kc *KubeClient) GetNodeIPs(nodeName string) (string, string, error) {
    // First attempt
    externalIP, internalIP, err := kc.getNodeIPsOnce(nodeName)
    if err != nil && isConnectionError(err) {
        // If connection error, try to reconnect once and retry
        log.Printf("Connection error detected: %v. Attempting to reconnect...", err)
        if reconnectErr := kc.reconnect(); reconnectErr != nil {
            return "", "", fmt.Errorf("failed to reconnect: %v, original error: %v", reconnectErr, err)
        }

        // Retry after reconnection
        externalIP, internalIP, err = kc.getNodeIPsOnce(nodeName)
        if err != nil {
            return "", "", fmt.Errorf("failed after reconnection: %v", err)
        }
    } else if err != nil {
        return "", "", err
    }

    return externalIP, internalIP, nil
}

// getNodeIPsOnce performs a single attempt to get node IPs
func (kc *KubeClient) getNodeIPsOnce(nodeName string) (string, string, error) {
    kc.mutex.RLock()
    clientset := kc.clientset
    kc.mutex.RUnlock()

    // Get the node by name
    node, err := clientset.CoreV1().Nodes().Get(context.TODO(), nodeName, metav1.GetOptions{})
    if err != nil {
        return "", "", fmt.Errorf("failed to get node %s: %v", nodeName, err)
    }

    var externalIP, internalIP string

    // Iterate through node addresses to find external and internal IPs
    for _, addr := range node.Status.Addresses {
        switch addr.Type {
        case corev1.NodeExternalIP:
            externalIP = addr.Address
        case corev1.NodeInternalIP:
            internalIP = addr.Address
        }
    }

    return externalIP, internalIP, nil
}

// nodeIPHandler handles the GET /node/ip endpoint
func (kc *KubeClient) nodeIPHandler(w http.ResponseWriter, r *http.Request) {
    // Set response content type to JSON
    w.Header().Set("Content-Type", "application/json")

    // Only allow GET method
    if r.Method != http.MethodGet {
        w.WriteHeader(http.StatusMethodNotAllowed)
        json.NewEncoder(w).Encode(NodeIPResponse{
            Error: "Method not allowed, only GET is supported",
        })
        return
    }

    // Get node name from query parameter
    nodeName := r.URL.Query().Get("name")
    if nodeName == "" {
        w.WriteHeader(http.StatusBadRequest)
        json.NewEncoder(w).Encode(NodeIPResponse{
            Error: "Missing required query parameter: name",
        })
        return
    }

    // Get node IPs from Kubernetes API
    externalIP, internalIP, err := kc.GetNodeIPs(nodeName)
    if err != nil {
        w.WriteHeader(http.StatusNotFound)
        json.NewEncoder(w).Encode(NodeIPResponse{
            NodeName: nodeName,
            Error:    err.Error(),
        })
        return
    }

    // Return successful response
    response := NodeIPResponse{
        NodeName:   nodeName,
        ExternalIP: externalIP,
        InternalIP: internalIP,
    }

    w.WriteHeader(http.StatusOK)
    json.NewEncoder(w).Encode(response)
}

// healthHandler provides a simple health check endpoint
func healthHandler(w http.ResponseWriter, r *http.Request) {
    w.Header().Set("Content-Type", "application/json")
    w.WriteHeader(http.StatusOK)
    json.NewEncoder(w).Encode(map[string]string{
        "status": "healthy",
    })
}

func main() {
    // Initialize Kubernetes client
    kubeClient, err := NewKubeClient()
    if err != nil {
        log.Fatalf("Failed to initialize Kubernetes client: %v", err)
    }

    log.Println("Successfully connected to Kubernetes cluster")

    // Set up HTTP routes
    http.HandleFunc("/node/ip", kubeClient.nodeIPHandler)
    http.HandleFunc("/health", healthHandler)

    // Start HTTP server
    port := "6464"
    log.Printf("Starting HTTP server on port %s", port)
    log.Printf("Endpoint: GET /node/ip?name=<node-name>")
    log.Printf("Health check: GET /health")

    if err := http.ListenAndServe(":"+port, nil); err != nil {
        log.Fatalf("Failed to start HTTP server: %v", err)
    }
}