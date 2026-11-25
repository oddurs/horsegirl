# The MCP server

`horsegirl mcp` serves four tools over MCP: `spec`, `check`, `fix` and `draw`.
An agent in an MCP client calls them and reads JSON. It does not run a shell
and does not parse terminal text.

## Add the server to a client

The client starts `horsegirl mcp` as a child process and talks to it on
standard input and standard output. The server listens on no port.

`horsegirl mcp --config` prints the entry for the configuration of a client.
The entry names the path of the binary that printed it:

```json
{
  "mcpServers": {
    "horsegirl": {
      "args": ["mcp"],
      "command": "/home/you/.cargo/bin/horsegirl"
    }
  }
}
```

The server speaks both eras of the protocol. A client of `2025-11-25` or
earlier sends `initialize`. A client of `2026-07-28` sends `server/discover`
and puts `_meta` on each request. The server is named `horsegirl` in
`serverInfo`.

## The tools

Each tool takes the whole text of one file, not a path:

```json
{ "source": "Web app -> API\n" }
```

`spec` takes no argument.

| Tool | Returns |
| --- | --- |
| `spec` | The language card, as one text block. |
| `check` | Every diagnostic of `source`, each with its fix, and the count of each severity. |
| `fix` | `source` with every safe fix made, the number of fixes made, and every diagnostic that is left. |
| `draw` | The picture as SVG, and every diagnostic of `source`. |

`check`, `fix` and `draw` return their result as `structuredContent`, and
each declares the matching `outputSchema`. The result also holds one text
block with the same JSON. The keys of every object are sorted by name.

### `check`

```json
{
  "diagnostics": [ diagnostic, ... ],
  "errors": 1,
  "notes": 0,
  "warnings": 0
}
```

A diagnostic is the same object that `horsegirl check --json` prints on one
line, and [Diagnostics](diagnostics.md#the-json-form) states its shape: `code`,
`fix`, `message`, `severity` and `span`. A test holds the two to the same
bytes. The diagnostics are sorted by the start of their span and then by code.
One call returns every diagnostic, so an agent can stop when `errors` is 0
without reading the array.

### `fix`

```json
{
  "applied": 2,
  "diagnostics": [ diagnostic, ... ],
  "errors": 0,
  "notes": 0,
  "source": "Web app -> \"API@v2\"\n",
  "warnings": 0
}
```

`source` is the text with every safe fix made and no other byte changed. As
`horsegirl fix` does, the tool checks again after each pass, until a pass
makes no fix. `applied` counts the fixes. `diagnostics` and the counts are
what is left in the new `source`. A suggested fix is left for the agent to
decide.

### `draw`

```json
{
  "diagnostics": [ diagnostic, ... ],
  "errors": 0,
  "notes": 0,
  "svg": "<svg xmlns=\"http://www.w3.org/2000/svg\" ...",
  "warnings": 0
}
```

`draw` draws through the lens of the first `show` line, or through flow when
the file has none. `svg` is the same text that `horsegirl draw` prints for
standard input. A file with an error gives no picture: `svg` is `null`, and
the diagnostics say what to fix.

## When a tool cannot run

A diagnostic is a result, not a failure. `isError` is `false` when the tool
ran, also when the file has errors.

`isError` is `true`, with the reason in one text block and no
`structuredContent`, when the tool cannot run:

- An argument is missing, such as `source`.
- `draw` is given a clean file whose `show` line names a lens that this
  version does not draw. The text names the lenses it draws.

A call to a tool that does not exist is a JSON-RPC error with the code
`-32602`.
