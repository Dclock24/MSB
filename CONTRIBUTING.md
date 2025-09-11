# Contributing

- Use feature branches and PRs; CI must pass.
- Go code: `gofmt`/`golangci-lint` (if added), explicit error handling.
- Julia: keep API calls cached/rate-limited; clamp metrics; no panics.
- No secrets in code or logs; use `.env.example`.
- Add/update docs under `docs/` for any public interface.
