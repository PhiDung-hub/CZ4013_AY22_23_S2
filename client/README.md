# cLient implementation

Run from main or this file with 
```bash
cargo run --bin client [-- -<options>]
```

Available options are:
+ addr: address that this is binded on, default to "127.0.0.1" (shorthand `-a`)
+ port: port that this client is binded on, default to 3000 (shorthand `-p`)
+ server-addr: address of the target server is binded on, default to "127.0.0.1"
+ server-port: port of the target server is binded on, default to 1234
+ loss: whether loss response is stimulate, default to false (shorthand `-l`)
+ loss-prob: probability of loss response, default = 25%
+ retry: use indefinite retry (at-least-one) invocation semantic, default = false (at-most-once)

For example (with loss of request and at-least-one invocation semantic enable):
```bash
cargo run --bin client -- --server-addr "127.0.0.1" --server-port 1234 -p 3000 -l --loss-prob 0.25 -r
```
