# stdscope

`stdscope` is a transparent proxy for inspecting a program over stdio transport.

It records and forwards stdin and stdout for a child process.

## Usage

```sh
# Wrap `sed` to capture standard input / output
$ echo "Hello world" | stdscope -- sed "s/Hello/Goodbye/"
--> Hello world
Goodbye world
<-- Goodbye world

# `-e` to write captured output to stderr
$ echo "Hello world" | stdscope -e -- sed "s/Hello/Goodbye/" >/dev/null
--> Hello world
<-- Goodbye world


# `-o FILE` to capture output to a file
$ echo '{"jsonrpc": "2.0", "method": "subtract", "params": {"x": 10, "y": 5}, "id": 3}' | stdscope -o ~/stdscope.log -- jsonrpc_calc
{"jsonrpc": "2.0", "result": 19, "id": 3}
$ cat ~/stdscope.log
--> {"jsonrpc": "2.0", "method": "subtract", "params": {"x": 25, "y": 6}, "id": 3}
<-- {"jsonrpc": "2.0", "result": 19, "id": 3}
```
