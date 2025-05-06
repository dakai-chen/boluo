# unreleased

## 变化

- 重构：优化服务器的实现和代码结构。
- 重构：将 `match_method` 提取为 `MethodRouter` 的方法以优化代码结构。
- 重构：修改 `add_endpoint` 和 `add_endpoint_with` 的实现，减少代码膨胀。
- 修改 `Server` 的 `Debug` 实现。

# 0.6.5

## 变化

- 重构：优化路由器的实现和代码结构。

# 0.6.4

## 新增

- 新增函数 `Router::scope_merge`，`Router::scope_merge_with`，`Router::try_scope_merge`，`Router::try_scope_merge_with`，允许在合并路由器时向所有路由添加前缀。

## 修复

- 恢复路径通配符替换逻辑为仅处理末尾的 {*}。

# 0.6.3

## 新增

- 新增函数 `Router::iter`，用于遍历路由器中的所有路由。
- 新增函数 `Router::remove`，用于移除路由器中的指定路由。
- 公开 `Endpoint` 类型，改进 `Router::iter` 函数。

# 0.6.2

## 变化

- `Router` 的 `merge`，`merge_with`，`try_merge`，`try_merge_with` 方法现在接受任何可以转换为 `Router` 的类型。

# 0.6.1

## 修复

- 修复嵌套路由路径 `{*}` 前缀缺失 `/` 的问题。

# 0.6.0

## 破坏

- 迁移到 rust 2024 (1.85.0) 版本。
- 改进 `Listener`，现在可以返回自定义的连接地址类型。
- 将模块 fs 重命名为 static_file，并将功能修改为使用 static-file 特性开关。
- 调整 features。
- 将 tokio-tungstenite 依赖的版本提升到 0.26，并重构 ws 模块。

## 新增

- 新增模块 upgrade。
- 新增函数 `KeepAlive::event`，用于自定义 SSE 保持连接的消息事件。

## 变化

- 使用 upgrade 模块完成 WebSocket 升级。

# 0.5.4

## 新增

- 公开 `PathParams`。

## 修复

- 修复少部分情况下 `Router` 在未找到处理程序时提取路径参数。

# 0.5.3

## 变化

- 重构 `GracefulShutdown` 的实现。

# 0.5.2

## 变化

- `Server::http1_header_read_timeout` 可以传递None以禁用配置。

## 修复

- `Server` 内部未配置 `Timer` 导致的 `http1_header_read_timeout` 等函数配置后服务器无法处理请求。

# 0.5.1

## 新增

- `Server` 新增函数：`http1_max_headers`，`http2_max_pending_accept_reset_streams`。

## 变化

- tokio-tungstenite = "0.24"。

# 0.5.0

## 破坏

- boluo-core = "0.4"。
- 修改 `ServeFile` 和 `ServeDir` 的 `Service` 实现。
- 修改 `ExtensionService` 的 `Service` 实现。
- 删除 `OptionalTypedHeader`。

## 新增

- 为 `Extension` 实现 `OptionalFromRequest`。
- 为 `TypedHeader` 实现 `OptionalFromRequest`。

# 0.4.0

## 破坏

- boluo-core = "0.3"。
- 重构模块 `boluo::extract::header`。

## 新增

- 导出 `headers` 库。

# 0.3.2

## 新增

- 为 `EventBuilder` 实现 `Default`。

# 0.3.1

## 修复

- 当 `http2` 功能关闭时，从依赖项中删除 `h2`。

# 0.3.0

## 破坏

- boluo-core = "0.2"。
- 移除 `FormResponseError` 的 `From<serde_urlencoded::ser::Error>` 实现。
- 移除 `JsonResponseError` 的 `From<serde_json::Error>` 实现。
- 重构错误类型：`MultipartError`，`RedirectUriError`，`EventValueError`。

## 新增

- 为 `PathExtractError` 实现 `Clone`。

## 变化

- 变化：移除实现 `Service` 的多余约束：`ServeFile`，`ServeDir`，`ExtensionService`。

# 0.2.0

## 破坏

- 私有化 `MethodRoute::any` 和 `MethodRoute::one`。
- 重构模块 `boluo::extract` 下的错误类型。
- 重构模块 `boluo::response` 下的错误类型。
- 变更提取器 `Form` 的行为，`HEAD` 请求也将从查询字符串中读取表单数据。
- `Router` 的嵌套路径不允许为空。

## 新增

- 为 `Extension` 实现 `IntoResponseParts`。
- 添加文档。

# 0.1.1

## 修复

- 将 `boluo/README.md` 的示例链接指向正确的位置。

# 0.1.0

- 初始发布