## Setup

Make sure you launch the proxy_server using cargo run within the directory. Afterwards, you can query the server for the data you want. The IP will automatically be localhost, all you must do is specify the port on which the server is running and the origin request you wish to make to a server. The proxy server will handle the rest and send back a JSON object that is parsed and printed within the command line.

## Example Commands to query the proxy Server

Simple Generic Query:

```
cargo run -- --port 3000 --origin https://pokeapi.co/api/v2/pokemon
```

Specific Query:

```
cargo run -- --port 3000 --origin https://pokeapi.co/api/v2/pokemon/ditto
```

Clear the proxy's cache:

```
cargo run -- --clear-cache
```
