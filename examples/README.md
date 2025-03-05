# 示例

这些示例演示了 `boluo` 的主要功能和使用方式，你可以通过以下命令运行这些示例：

```bash
cargo run --bin [crate] # [crate]替换为具体的示例项目
```

| 目录                                      | 说明                                       |
| ----------------------------------------- | ------------------------------------------ |
| [hello](./hello/)                         | 输出 "hello world"                         |
| [route](./route/)                         | 添加路由，为处理程序设置访问路径和访问方法 |
| [state](./state/)                         | 添加状态，用于在处理程序中共享资源         |
| [extract-path](./extract-path/)           | 提取路径参数                               |
| [handle-error](./handle-error/)           | 捕获错误，并将错误转为响应                 |
| [custom-middleware](./custom-middleware/) | 自定义中间件，并将中间件挂载到服务上       |
| [custom-listener](./custom-listener)      | 自定义监听器                               |
| [graceful-shutdown](./graceful-shutdown/) | 优雅关机                                   |
| [sse](./sse/)                             | 服务器发送事件                             |
| [ws](./ws/)                               | 网络套接字                                 |
| [log](./log/)                             | 记录请求日志                               |
| [static-file](./static-file/)             | 静态文件服务                               |
| [tls](./tls/)                             | 配置 TLS 加密以增强安全性                  |
| [compat-tower](./compat-tower/)           | 使用 `tower` 的服务和中间件                |
