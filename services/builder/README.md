# Builder

This is responsible for building the app, it uses nixpacks to build and create a docker image out of the users codebase

## API

/build
```
curl -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "path": "/path/to/build/",
    "name": "example-image",
    "envs": ["ENV_VAR1=value1", "ENV_VAR2=value2"],
    "build_options": {
      "name": "example-image",
      "tags": ["latest"],
      "quiet": true
    }
  }' \
  http://localhost:8084/build
```

/logs
```
curl -X GET \
  "http://localhost:8084/logs?container_id=<container_id>&start_time=<start_time>&end_time=<end_time>"
```