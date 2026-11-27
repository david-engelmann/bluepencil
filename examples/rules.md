# Repository rules

Read these instructions before modifying the workspace.

- Run the verification gate before opening a pull request
- Never commit directly to main
- Keep dependencies minimal

## Testing

- Prefer deterministic unit tests over network mocks
  including offline fixture coverage
- Assert exact exit codes in CLI tests

```markdown
- Bullets inside fenced blocks are examples, not active rules
```

## Architecture

- Validate external JSON at module boundaries
- Keep error messages actionable and free of stack traces
