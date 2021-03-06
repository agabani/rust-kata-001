# Edge Cases

https://crates.io/api/v1/crates/syn/0.11.0/dependencies
RFC invalid versions -> "0.*"
```
{
    "id": 155844,
    "version_id": 43170,
    "crate_id": "clippy",
    "req": "0.*",
    "optional": true,
    "default_features": true,
    "features": [],
    "target": null,
    "kind": "normal",
    "downloads": 0
},
```

https://crates.io/api/v1/crates/rand/0.7.3/dependencies
Kinds
```
{
    "id": 1082603,
    "version_id": 202916,
    "crate_id": "rand_hc",
    "req": "^0.2",
    "optional": false,
    "default_features": true,
    "features": [],
    "target": "cfg(target_os = \"emscripten\")",
    "kind": "normal",
    "downloads": 0
},
{
    "id": 1082600,
    "version_id": 202916,
    "crate_id": "rand_hc",
    "req": "^0.2",
    "optional": false,
    "default_features": true,
    "features": [],
    "target": null,
    "kind": "dev",
    "downloads": 0
},
```

https://crates.io/api/v1/crates/net2/0.2.29/dependencies
Targets
```
{
    "id": 199628,
    "version_id": 52235,
    "crate_id": "libc",
    "req": "^0.2.14",
    "optional": false,
    "default_features": true,
    "features": [],
    "target": "i686-unknown-linux-gnu",
    "kind": "normal",
    "downloads": 0
},
{
    "id": 199627,
    "version_id": 52235,
    "crate_id": "libc",
    "req": "^0.2.14",
    "optional": false,
    "default_features": true,
    "features": [],
    "target": "x86_64-unknown-linux-gnu",
    "kind": "normal",
    "downloads": 0
},
{
    "id": 199622,
    "version_id": 52235,
    "crate_id": "libc",
    "req": "^0.2.14",
    "optional": false,
    "default_features": true,
    "features": [],
    "target": "x86_64-apple-darwin",
    "kind": "normal",
    "downloads": 0
},
{
    "id": 199621,
    "version_id": 52235,
    "crate_id": "libc",
    "req": "^0.2.14",
    "optional": false,
    "default_features": true,
    "features": [],
    "target": "cfg(unix)",
    "kind": "normal",
    "downloads": 0
},
{
    "id": 199623,
    "version_id": 52235,
    "crate_id": "libc",
    "req": "^0.2.14",
    "optional": false,
    "default_features": true,
    "features": [],
    "target": "i686-apple-darwin",
    "kind": "normal",
    "downloads": 0
},
```

https://crates.io/api/v1/crates/yaml-rust/0.3.5/dependencies
Multi req
```
{
    "dependencies": [
        {
            "id": 133310,
            "version_id": 38451,
            "crate_id": "clippy",
            "req": "0.*",
            "optional": true,
            "default_features": true,
            "features": [],
            "target": null,
            "kind": "normal",
            "downloads": 0
        },
        {
            "id": 133309,
            "version_id": 38451,
            "crate_id": "linked-hash-map",
            "req": ">=0.0.9, <0.4",
            "optional": true,
            "default_features": true,
            "features": [],
            "target": null,
            "kind": "normal",
            "downloads": 0
        }
    ]
}
```
