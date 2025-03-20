# stdscope

`stdscope` is a transparent proxy for inspecting a program over stdio transport.
It records and forwards stdin, stdout, and stderr for a child process.

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

# Capture both stdout and stderr
$ echo "test" | stdscope -- sh -c 'echo "to stdout"; echo "to stderr" >&2'
--> test
to stdout
<-- to stdout
to stderr
!-- to stderr
```

## Example: Debugging MCP Servers

```sh
# Intercept MCP messages between MCP client and server
$ npx @modelcontextprotocol/inspector@latest -- stdscope -o $HOME/mcp.log -- npx @modelcontextprotocol/server-everything

# Tail the log in another terminal:
$ tail -f ~/mcp.log
--> {"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2024-11-05",...}}
<-- {"result":{"protocolVersion":"2024-11-05","capabilities":{"prompts":{}},...},"id":0}
--> {"jsonrpc":"2.0","method":"notifications/initialized"}
--> {"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}
<-- {"result":{"tools":[{"name":"echo","description":"Echoes back the input"},...]},"id":1}
<-- {"method":"notifications/message","params":{"level":"info","data":"Info-level message"},"jsonrpc":"2.0"}
```