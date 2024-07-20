# greenhouse_backend
## Build docker containers 
### Example smart device 
```console
docker build -t <TAG> --build-arg RUST_VERSION=1.78.0 -f examples/hybrid_device/. . --no-cache
```