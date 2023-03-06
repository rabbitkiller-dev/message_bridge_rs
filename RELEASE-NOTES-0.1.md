# message_bridge_rs 0.1

## message_bridge_rs 0.1.1

THIS IS NOT RELEASED YET

### Changes since message_bridge_rs 0.1.0

* feat: Support forward Tencent QQ group messages to Discord
* feat: Complete bridge encapsulation
* feat: Implement forward bridge messages to Tencent QQ groups
* feat: Add bridge user and log related stuffs
* feat: Support send user avatar when forwarding message
* (bug #2) feat: Add command 'channel'
* feat: Support convert Tencent QQ group message images to bridge images and
  forward bridge images to Discord
* feat: Support forward Discord image attachments to Tencent QQ group
* feat: Implement read & write user binding data (sync)
* perf: Extend bridge::User
* fix: Fix cmd_adapter errors
* feat: Add Discord and Tencent QQ group user query and tests
* refactor: Adjust formats for map-type data
* (bug #3) pref: Command enumeration
* (bug #4) feat: Add command 'confirm bind'
* pref: Split message handling
* (bug #4) pref: Unify user ID type
* feat: Parse Discord message and identify cross-platform user mentions by using
  JS library
* (bug #4) feat: Update command 'bind'
* (bug #4) pref: Improve data structures for map-type data
* fix: Temporarily disable command responses
* feat: Support mentioning Discord users from Tencent QQ grops
* (bug #5) feat: Sync CMD command responses
* ci: Create rust.yml
* feat: Support context for Mirai events
* (bug #4) fix: Check, create data directories before read/write binding data
* (bug #4) fix: Return failure-type responses when failed to save binding data
* (bug #4) pref: Simplify query parameters for user binding queries
* ci: Create main.yml
* ci: Update rust.yml - remove 'cargo test'
* ci: Update rust.yml - add 'upload-artifact'
* ci: Test builds
* ci: Use releases mode for builds
* ci: Upload build artifacts
* feat: Use tracing to record logs
* pref: Add, adjust tracing logs for bridge
* feat: Add dependency 'lazy_static'
* feat: Add bridge message history
* feat: Record relationships between sent messages and forward messages
* (bug #4) feat: Add command 'unbind'
* feat: Implement BitOr (bitwise operation 'OR') for platform enumeration
* feat: Change reqwest - use rustls
* pref: Replace println to logs
* (bug #8) docs: Add COPYING, CREDITS, HISTORY, RELEASE-NOTES, SECURITY

## message_bridge_rs 0.1.0

This is the version of the initial commit of message_bridge_rs.

### Changes of the initial commit of message_bridge_rs

* feat: Add Mirai verify, bind APIs
