# Examples

- [`restart-dependency`](./restart-dependency):
  A HTTP API depends on another service and allows restarting it.
  During this process, the API returns `503 Service Unavailable`.
  It also allows hot-reloading its configuration, serving
  `500 Internal Server Error` if the configuration is incorrect
  until a successful reload.
