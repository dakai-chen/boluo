# unreleased

## 新增

- 新增方法 `Body::to_bytes`。

# 0.5.1

## 修复

- 修复 `Upgraded::downcast` 无法向下转型。

# 0.5.0

## 破坏

- 迁移到 rust 2024 (1.85.0) 版本。
- 使用 `ServiceExt::with` 挂载中间件，结果必须是一个 `Service`。

## 新增

- 新增模块 upgrade。

# 0.4.1

## 新增

- 添加 `simple_middleware_fn` 和 `simple_middleware_fn_with_state` 函数。

# 0.4.0

## 破坏

- 修改 `Option<T>` 的 `FromRequest` 实现。

## 新增

- 添加 `OptionalFromRequest` 特征。
- 添加 `ExtractResult`，简化 `Result` 提取器的书写。

# 0.3.0

## 破坏

- 删除特征 `boluo_core::extract::Name`。
- 删除宏 `boluo_core::name`。

# 0.2.1

## 新增

- 允许处理程序参数末尾提取 `Request`。

# 0.2.0

## 破坏

- 重命名 `IntoHeaderError` 为 `HeaderResponseError`，并重构其实现。
- 修改 `ServiceExt` 特征，对除了 `with` 以外的函数添加更多约束，保证函数返回的对象是 `Service`。

## 变化

- 移除实现 `Service` 的多余约束：`Then`，`AndThen`，`OrElse`，`MapResult`，`MapResponse`，`MapErr`，`MapRequest`，`ServiceFn`，`BoxService`，`BoxCloneService`，`ArcService`。

# 0.1.1

## 新增

- 添加文档。

# 0.1.0

- 初始发布