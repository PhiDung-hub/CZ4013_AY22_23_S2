# Server implementation

Run from main or this file with 
```bash
cargo run --bin server [-- -<options>]

```

## Seed Database 
```bash
cargo run --bin seed_db
```
Then move the `flight_informations.db` file to root folder.


Available options are:
+ address: that this server is binded to, default to 127.0.0.1 (short hand `-a`)
+ port: that this server is binded on, default to 1234 (short hand `-p`)
+ loss: whether loss response is stimulate, default false (short hand `-l`)
+ loss-prob: probability of loss response, default = 25%

For example (with loss enable):
```bash
cargo run --bin server -- --adr 127.0.0.1 --port 1234 --loss --loss-prob 0.25
```
