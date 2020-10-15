# Default example setup to demo SMTP Filter

## Files

| File              | Description              | Purpose                                                                 |
| ----------------- | ------------------------ | ----------------------------------------------------------------------- |
| `example.yaml`    | `Example` descriptor     | Describes runtime requirements, e.g. a specific version of `Envoy`      |
| `envoy.tmpl.yaml` | `Envoy` bootstrap config | Provides `Envoy` config that demoes extension in action                 |
| `extension.json`  | `Extension` config       | Provides configuration for extension itself                             |

## Components

### Envoy config

#### Listeners

* [0.0.0.0:10000](http://0.0.0.0:10000) - represents a TCP ingress
  * proxies traffic to a `SMTP Server` (you need to run it yourself)
  * configured to use `SMTP Filter` extension

### Extension config

To see detailed stats per SMTP verb and reply code, use

```json
{
    "detailed_stats": true
}
```

## Request Flow

```
+-------------+                  +----------------------+              +---------------------------+
|             |  (SMTP session)  | Envoy (SMTP Filter)  | (dispatches) |        SMTP Server        |
| SMTP client | ---------------> |                      | -----------> |                           |
|             |                  | http://0.0.0.0:10000 |              |   http://127.0.0.1:1025   |
+-------------+                  +----------------------+              +---------------------------+
                                           | (uses)
                                           V
                                +------------------------+
                                |    SMTP Filter (Wasm)  |
                                +------------------------+
```

## How to use

1. Setup SMTP server on `127.0.0.1:1025`
   * E.g., use https://github.com/maildev/maildev
2. Setup SMTP client
   * E.g., a Java application with `log4j` configured to log to SMTP
3. Checkout `Envoy` stats
