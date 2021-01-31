# railroute
A simple, blazing fast TCP router.

# Example
```
> railroute -p 8000 -a google.com:80
19:03:18 [INFO] Server listening on port 8000
19:03:21 [INFO] Accepted connection from 127.0.0.1:42316
19:03:24 [INFO] Connection from 127.0.0.1:42316 terminated

```

# Usage
Usage information can be found on the help page.

```
railroute 1.0
UE2020
A simple, blazing fast TCP router.

USAGE:
    railroute [OPTIONS] --address <ADDRESS>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --address <ADDRESS>    Sets the routed address
    -p, --port <PORT>          Sets a custom port (default is 3000)
```

