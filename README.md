# Infralink

Infralink is a container orchestration tool that is fast & simple.

## Breakdown

This explains what each service does in Infralink

`principal`

This manages volumes on cloud platforms, pre-warmed instances defined by rules in a database and receives metrics from the worker. 

`worker`

The worker has logic to create containers, modify them, get statistics from the containers (cpu, memory, network). It also automatically heals containers, and supports selecting healing, and rolling updates. 

`builder`

The builder takes a local path or a git repository and builds it using (nixpacks)[https://nixpacks.com], after that it pushes the image to a registry. 

`runner (REST)`

This pulls an image from a registry and then runs it as a container.

`scaler`

This manages auto-scaling/scaling for containers & cloud instances, it checks certain statistics and if it passes a certain threshold it will scale up containers/instance.

`registry`

Manages pulls/pushes to a registry.