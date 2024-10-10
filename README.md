# k8s-job-webhooks

`k8s-job-webhooks` is a Kubernetes client application built with Rust that monitors Kubernetes Jobs and triggers webhooks upon Job completion.

The application exposes an HTTP server and REST endpoints built with Actix Web, allowing users to create webhooks that
are triggered upon the completion of specific Jobs.

This project is currently in development.
## REST endpoint
- ```json
  POST /cronjobs/monitors
  {
      "cronjobs": ["cronjob-name-1", "cronjob-name-2", "cronjob-name-3"],
      "webhooks": [
        {
          "url": "...",
          "requestBody": "..."
        }
      ]
  }
- ```json
  POST /jobs/monitors
  {
      "jobs": ["job-name-1", "job-name-2", "job-name-3"],
      "webhooks": [
        {
          "url": "...",
          "requestBody": "..."
        }
      ]
  }
   ```
## How to use it effectively
The best way to use `k8s-job-webhooks` is to create Kubernetes jobs that include init containers to register webhooks. Here is an example of a job definition:
```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: example-job
spec:
  template:
    spec:
      initContainers:
        - name: init-http-request
          image: curlimages/curl:7.85.0
          command:
            - "sh"
            - "-c"
            - |
              curl -v -X POST http://k8s-job-monitor-service:8080/jobs/monitors \
              -H 'Content-Type: application/json' \
              -d '{
                    "jobs": ["example-job"],
                    "webhooks": [
                      {
                        "url": "...",
                        "requestBody": "..."
                      }
                    ]
                  }'
      containers:
        - name: example-container
          image: busybox
          command: ["echo", "Hello, Kubernetes!"]
      restartPolicy: Never
  backoffLimit: 4
```