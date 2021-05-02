# POD-ENV
Print pod environment variables and resolve secret.

## Run
```bash
NAMESPACE=kube-system cargo run
```

## Debug
```bash
RUST_LOG=debug cargo run
```

### Minikube workaround
Add an ip entry (here 192.168.64.3 localhost) to /etc/hosts, then and search replace the ip with localhost in ~/.kube/config