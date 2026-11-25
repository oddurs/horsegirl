# MCP prototype (SPIKE-21)

A throwaway stdio MCP server with one tool, `check`. It is not part of the
horsegirl crate, and `scripts/task` does not build it. The answer in
`cairn/items/SPIKE-21-*.md` explains what it showed.

Build it, test it against the golden result, and call it from the reference
client:

```sh
cd spikes/mcp
cargo build
cargo test
npx @modelcontextprotocol/inspector@2.16.0 --cli ./target/debug/horsegirl-mcp-spike \
  --method tools/call --tool-name check --tool-arg 'source=Café --> Payments'
```
