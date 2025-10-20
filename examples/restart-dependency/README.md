# Requirements

- Base routes
  - `GET /health`: Return API status
- API crash if bad config file at startup
- Reload config file at runtime (`POST /reload`)
  - Regenerates app state
  - Serve `500 Internal Server Error` if bad configuration
- `POST /restart-dependency`: Fake dependency that takes some time to restart
  - Serve `503 Service Unavailable` during restart
- `GET /users`: Works only if dependency available
