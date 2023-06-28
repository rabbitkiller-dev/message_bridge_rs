# message_bridge_rs 1.0

## message_bridge_rs 1.0.0-alpha.4

THIS IS NOT A RELEASE YET

### Changes since message_bridge_rs 1.0.0-alpha.3

* (bug #17) docs: Update RELEASE-NOTES for 1.0.0-alpha.1, 1.0.0-alpha.2,
  1.0.0-alpha.3
* (bug #18) Upgrade dependency 'proc_qq'
* fix: Fix missing QQ user nickname
* (bug #20) feat: Add bridge relay for Telegram
* (bug #21) feat: 增加桥消息管理, 序列化存储桥消息&重构发送消息的操作 - part 1
* (bug #21) feat: 增加桥消息管理, 序列化存储桥消息&重构发送消息的操作 - part 2
* (bug #21) feat: 增加桥消息管理, 序列化存储桥消息&重构发送消息的操作 - part 3
* (bug #21) feat: qq和dc平台的消息发送和接收同步都绑定到桥消息中
* (bug #21) feat: 完善回复消息功能 - part 1 (50%)
* (bug #21) feat: 完善回复消息功能 - part 2 (80%)
* (bug #21) feat: 完善回复消息功能 - part 3 (80%)
* (bug #21) feat: 调整rq配置
* (bug #21) rq使用配置来初始化
* (bug #21) feat: 完善回复消息功能 - part 4
* (bug #24) 调整模块位置
* (bug #24) feat: dc和qq支持跨平台提及at用户功能 - part 1
* (bug #24) feat: dc和qq支持跨平台提及at用户功能 - part 2
* (bug #30) feat: 使用 clap 库解析指令消息
* (bug #30) feat: 实现指令提示
* (bug #30) feat: 增加桥命令客户端
* (bug #30) 转移指令模块；调整结构
* (bug #30) feat: 重做关联指令(原绑定)
* (bug #33) doc: 增加文本说明
* (bug #33) feat(bridge_dc): 增加过滤消息方法
* (bug #34) fix(bridge): 修复查询另一个平台用用户逻辑错误
* (bug #36) doc: 更新dc链接&修改命令使用说明
* (bug #37) fix: Check脚本
* (bug #40) Update RELEASE-NOTES-1.0

## message_bridge_rs 1.0.0-alpha.3

message_bridge_rs 1.0.0-alpha.3 is an alpha-quality development branch.

See bug #15, #16.

### Changes since message_bridge_rs 1.0.0-alpha.2

* feat: Handle file extension issues - part 2
* feat: Handle file extension issues - part 3

## message_bridge_rs 1.0.0-alpha.2

message_bridge_rs 1.0.0-alpha.2 is an alpha-quality development branch.

See bug #14.

### Changes since message_bridge_rs 1.0.0-alpha.1

* (bug #7) fix: Fix CI 'act-build'
* (bug #8, #12) docs: Update RELEASE-NOTES

## message_bridge_rs 1.0.0-alpha.1

message_bridge_rs 1.0.0-alpha.1 is an alpha-quality development branch.

See bug #13.

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
* (bug #7) feat: Migrate QQ group functionalities from MCL to RICQ
* (bug #7) feat: Complete Tencent QQ group bridge & support improved message
  images to bridge - part 1
* (bug #7) feat: Complete Tencent QQ group bridge & support improved message
  images to bridge - part 2
* (bug #7) feat: Add user_manager for bridge user management
* (bug #4, #7) feat: Configure global service 'user_manager' & complete user
  mention handling in Discord and Tencent QQ groups
* (bug #4, #7) feat: Complete user relation query feature & implement fetch
  cross-platform user mention for bonded accounts for Discord and Tencent QQ
  group
* (bug #7) feat: Add binary expression macro
* (bug #7) feat: Adjust variable name & update module structures - part 1
* (bug #7) feat: Adjust variable name & update module structures - part 2
* (bug #7) docs: Update README.md
* (bug #7) feat: Remove feature 'message history'
* (bug #7) fix: Log level too low when not being set in environment varibles
* (bug #7) pref: Clean up modules
* (bug #7) fix: Use nightly for CI 'act-build' for GitHub
* feat: Handle file extension issues
