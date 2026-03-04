# ettac

## Running tests

Before running `docker compose` commands, for the first time it is recommanded to manually build the `ettac-ssh` image as it is used twice in the docker compose file and will result in an error on first launch. You can run the following command:
```shell
docker build --tag ettac-ssh:latest ./docker/ssh  
```

### Integration testing

Run the following command:
```shell
docker compose --project-directory docker up with-password with-public-key --detach
cargo test --features integration
```

### Image testing

The CI has a `test-image` job that spins up two SSH servers and tests the built image against them. This test can be reproduced locally by running the following command:
```shell
docker compose --project-directory docker up --abort-on-container-exit --exit-code-from test
```

You can also create a `docker/docker-compose.override.yml` file to use custom arguments or a different deploy script:
```yaml
name: ettac

services:
  test:
    command: ["with-password", "with-public-key"]
    volumes:
      - ../examples/empty.lua:/deploy.lua
```
