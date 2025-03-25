# CHANGELOG

## 3.6.1

### Fixed

- add validation for service creation (#111)
- fix serval ui bugs
- sync code editor theme with platform theme

### User Changes

- wsrx ui optimization (#181)
- improve update alert (#178)
- user with edit permissions can upload unlimited images to media API
- mobile view for all pages is available now (thanks to @Cnily03)
- redirect challenge management links to training when game is archived

### Common Developing Changes

- update @ark-ui/solid to `^5`, with bug fixes.

## 3.6.0

### BREAKING CHANGES

- No longer support to configure `cluster.cleanup_interval`, fixed to 30 seconds.
- Add archive policy for games and related configuration page. Now we can control the performance about archived challenges, whether to show hints or anwers for example. (#149)

### Fixed

- Fixes blockquote title highlight error in specific formats. (#104)
- Only display error tips when focusing, which avoid always hanging on and hiding other fields. (#150)
- Fixes where there is a probability taht unlocked tips cannot be loaded if refreshing at hint page directly. (#153)
- Fixes incorrect blood score calculation.

### Common Developing Changes

- Use `octet-stream` as restricted attachment type.
- Add refresh timers to notifications and chats.
- Better performance for popover of publish button
- Enhance the check at backend when unlocking hints.

### User Changes

- Randomly fill key when selecting checker preset. (#127)
- Challenge tabs now support middle mouse click to close.
- Support to display full log message when hovering.
- Add `Ctrl`+`Enter` shortcut for hammer chat
- Optimize some translations.
- Users now won't be logged out when update permissions of themselves.
- Add an input box for further confirmation when deleting users.

## 3.5.8

### BREAKING CHANGES

- Move rate limit config to independent section. When `rate_limit` section is missing, the rate limit feature for server will be disabled.

  ```diff toml
  # config.toml

   [server]
  -api_burst_limit        = 32
  -api_burst_restore_rate = 500             # in milliseconds

  +[server.rate_limit]
  +burst_limit        = 32
  +burst_restore_rate = 500                 # in milliseconds
  ```

## 3.5.7

### Common Developing Changes

- Using better default email html template in backend fallback. (#91)
- Add menu button for admin page on mobile view. (#106)
- Use `clsx` to construct component classes.
- Add time info on chat message.

### User Changes

- Increase dynamic score limit. Now the platform supports 0-1500 score ranges with max to 50 decays.

## 3.5.6

### BREAKING CHANGES

- Event API will always report challenge state changes.

### Fixed

- Setup nats consumer inactive state, which may fix email worker stuck issue.
- Delete pods when creating corresponding services failed. Partially fixes #111.
- Fixes cluster maintain worker, add server panic event to Event API.

### Common Developing Changes

- Migrate to tailwindcss v4.

### User Changes

- The service port in challenge environment configuration of challenge have state check now.

## 3.5.5

### Fixed

- [SECURITY] Traffic API through wsrx will check service ports before proxing.

### User Changes

- Support `ImagePullSecret` in challenge environment configuration.

## 3.5.4

### BREAKING CHANGES

- Use generic cell rate algorithm for rate limit.

  ```diff toml
  # config.toml

   [server]
  -api_rate_limit = 0
  +api_burst_limit        = 32
  +api_burst_restore_rate = 500             # in milliseconds
  ```
