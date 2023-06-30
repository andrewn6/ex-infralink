# Builder

This is responsible for building the app, it uses nixpacks to build and create a docker image out of the users codebase

## API

/build

*You can replace path below with a local directory*

```
curl -X POST -H "Content-Type: application/json" -d '{
  "path": "https://github.com/your-username/your-repo.git", 
  "name": "my-image",
  "envs": ["ENV_VAR1=value1", "ENV_VAR2=value2"],
  "build_options": {
    "name": "my-image",
    "tags": ["v1.0", "latest"]
  }
}' http://localhost:8084/build
```

/logs
```
curl -X GET \
  "http://localhost:8084/logs?container_id=<container_id>&start_time=<start_time>&end_time=<end_time>"
```