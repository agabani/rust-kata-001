# Endpoints

This document contains the list of supported endpoints.

## /dependency?name={name}&version={version}

Gets dependencies for crate+version

```
GET /dependency?name=quote&version=1.0.7

HTTP/1.1 200 OK
content-type: application/json

[
    {
        "name": "proc-macro2",
        "version": "1.0.0",
        "dependency": [
            {
                "name": "unicode-xid",
                "version": "0.2.0"
            }
        ]
    },
    {
        "name": "quote",
        "version": "1.0.7",
        "dependency": [
            {
                "name": "proc-macro2",
                "version": "1.0.0"
            }
        ]
    },
    {
        "name": "unicode-xid",
        "version": "0.2.0",
        "dependency": []
    }
]
```

## /health

Standardized health check ([Health Check Response RFC Draft for HTTP APIs](https://github.com/inadarei/rfc-healthcheck))

```
GET /health HTTP/1.1

HTTP/1.1 200 OK
content-type: application/health+json

{
    "status": "pass",
    "version": "0",
    "releaseId": "0.1.0",
    "description": "health of rust-kata-001 service",
    "checks": {
        "internet:http:connectivity": [
            {
                "componentType": "system",
                "status": "pass",
                "time": "2020-10-25T01:28:41Z"
            }
        ],
        "internet:https:connectivity": [
            {
                "componentType": "system",
                "status": "pass",
                "time": "2020-10-25T01:28:41Z"
            }
        ],
        "uptime": [
            {
                "componentType": "system",
                "status": "pass",
                "time": "2020-10-25T01:28:41Z"
            }
        ],
        "mysql:connectivity": [
            {
                "componentType": "datastore",
                "status": "pass",
                "time": "2020-10-25T01:28:41Z"
            }
        ]
    }
}
```