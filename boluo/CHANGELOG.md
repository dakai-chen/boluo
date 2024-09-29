# unreleased

- 变化：tokio-tungstenite = "0.23"。
- 新增：`Server`新增函数：`http1_max_headers`，`http2_max_pending_accept_reset_streams`。

# 0.5.0

- 破坏：boluo-core = "0.4"。
- 破坏：修改`ServeFile`和`ServeDir`的`Service`实现。
- 破坏：修改`ExtensionService`的`Service`实现。
- 破坏：删除`OptionalTypedHeader`。
- 新增：为`Extension`实现`OptionalFromRequest`。
- 新增：为`TypedHeader`实现`OptionalFromRequest`。

# 0.4.0

- 破坏：boluo-core = "0.3"。
- 破坏：重构模块`boluo::extract::header`。
- 新增：导出`headers`库。

# 0.3.2

- 新增：为`EventBuilder`实现`Default`。

# 0.3.1

- 修复：当`http2`功能关闭时，从依赖项中删除`h2`。

# 0.3.0

- 破坏：boluo-core = "0.2"。
- 破坏：移除`FormResponseError`的`From<serde_urlencoded::ser::Error>`实现。
- 破坏：移除`JsonResponseError`的`From<serde_json::Error>`实现。
- 破坏：重构错误类型：`MultipartError`，`RedirectUriError`，`EventValueError`。
- 新增：为`PathExtractError`实现`Clone`。
- 变化：移除实现`Service`的多余约束：`ServeFile`，`ServeDir`，`ExtensionService`。

# 0.2.0

- 破坏：私有化`MethodRoute::any`和`MethodRoute::one`。
- 破坏：重构模块`boluo::extract`下的错误类型。
- 破坏：重构模块`boluo::response`下的错误类型。
- 破坏：变更提取器`Form`的行为，`HEAD`请求也将从查询字符串中读取表单数据。
- 破坏：`Router`的嵌套路径不允许为空。
- 新增：为`Extension`实现`IntoResponseParts`。
- 新增：添加文档。

# 0.1.1

- 修复：将`boluo/README.md`的示例链接指向正确的位置。

# 0.1.0

初始版本