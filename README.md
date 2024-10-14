# k8s-job-webhooks

`k8s-job-webhooks` is a Kubernetes client application built with Rust that monitors Kubernetes Jobs and triggers webhooks upon Job completion.

The application exposes an HTTP server and REST endpoints built with Actix Web, allowing users to create webhooks that
are triggered upon the completion of specific Jobs.

This project is currently in development.
## REST endpoints
You can find the OpenAPI specification in the project directory.
- `POST /webhooks`
- `GET /webhooks`
- `POST /job-done-watchers`
- `GET /job-done-watchers/{id}`
- `GET /job-done-watchers`
## How to use it
Before using `k8s-job-webhooks`, you need to create at least one webhook using the `POST /webhooks` endpoint.

Once a webhook is created, you need to create a Job Done Watcher (observer completion Job). The best way
to automate this process is by using init containers within your Kubernetes Jobs.

To observe a Job for its completion, you must create a Job Done Watcher using the `POST /job-done-watchers` endpoint.
The request body requires the Job name and the webhook ID (obtained earlier).
The Job name can easily be injected into the init container using Kubernetes' downward API via environment variable.

Here is an example of a Kubernetes Job definition with an init container that registers a Job Done Watcher automatically:
The best way to use `k8s-job-webhooks` is to create Kubernetes jobs that include init containers to register webhooks. 
Here is an example of a job definition:
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
          image: busybox
          env:
            - name: JOB_NAME
              valueFrom:
                fieldRef:
                  fieldPath: metadata.labels['job-name']
          command:
            - "sh"
            - "-c"
            - |
              requestBody='{
                  "jobName": "$JOB_NAME",
                  "jobDoneTriggerWebhooks": [
                    {
                      "webhookId": "bb8d54c0-42f0-4d96-9e50-151645693a94"
                    }
                  ]
              }'

              requestBody=$(echo $requestBody | sed "s/\$JOB_NAME/$JOB_NAME/")

              wget --header="Content-Type: application/json" \
                --post-data="$requestBody" \
                http://k8s-job-webhook-service:8080/job-done-watchers
      containers:
        - name: example-container
          image: busybox
          command: ["printenv"]
      restartPolicy: Never
  backoffLimit: 4
```

## Using CronJob
The procedure for using CronJobs is quite similar to that for Jobs. However, since a CronJob may create multiple Pods
for the same Job, it’s important to avoid creating the same Job Done Watcher multiple times, which would result in multiple
webhook invocations upon Job completion. To prevent this, the `POST /job-done-watchers` endpoint allows you to specify
an `Idempotency-Key` HTTP header, making the request idempotent.

Here’s an example of a CronJob definition:
```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: example-cronjob
spec:
  schedule: "*/1 * * * *" 
  jobTemplate:
    spec:
      parallelism: 2 # => two init containers
      completions: 2  
      template:
        spec:
          initContainers:
            - name: init-http-request
              image: busybox
              env:
                - name: JOB_NAME
                  valueFrom:
                    fieldRef:
                      fieldPath: metadata.labels['job-name']
              command:
                - "sh"
                - "-c"
                - |
                  requestBody='{
                      "jobName": "$JOB_NAME",
                      "jobDoneTriggerWebhooks": [
                        {
                          "webhookId": "bb8d54c0-42f0-4d96-9e50-151645693a94",
                          "timeoutSeconds": 0
                        }
                      ]
                  }'

                  requestBody=$(echo $requestBody | sed "s/\$JOB_NAME/$JOB_NAME/")

                  wget --header="Content-Type: application/json" \
                    --header="Idempotency-Key: $JOB_NAME" \
                    --post-data="$requestBody" \
                    http://k8s-job-webhook-service:8080/job-done-watchers
          containers:
            - name: echo-container
              image: busybox
              command: ["printenv"]
          restartPolicy: OnFailure
```
## Job Labeling
The completed Job will be labeled with the label `app.k8s.job.webhooks/webhooks-called` set to true.