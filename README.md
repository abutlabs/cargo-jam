# cargo-jam
cargo, make me a JAM service


## Objective

Deliver a Rust Package Registry (crates.io) for bootstraping JAM Services. Our project should follow the golden standard [cargo-generate](https://github.com/cargo-generate/cargo-generate) project architecture while enabling users to quickly generate JAM Service projects. 

## Quick Commands 


```install (after publishing to crates.io)
cargo install cargo-jam
```

```publish new version from repo
cargo login <your-token>
cargo publish
```


```use it (create jam services in other projects)
cargo jam new <service-name>
```

## Sample JAM service

[ZK JAM Service](https://github.com/abutlabs/zk-jam-service) is Abutlabs 1st JAM service build. Use this project as reference to a sample build / deploy.