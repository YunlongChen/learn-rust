# Changelog

## 2025-08-27

### 修复
- **DNS记录加载功能修复**: 修改了 `handle_dns_reload` 方法，将其从调用同步的 `query_aliyun_dns_list` 函数改为异步地使用 `AliyunDnsClient` 的 `list_dns_records` 方法
  - 添加了阿里云认证信息的获取逻辑（从环境变量读取 `ALIYUN_ACCESS_KEY_ID` 和 `ALIYUN_ACCESS_KEY_SECRET`）
  - 增加了完整的错误处理机制
  - 修复了重复导入 `std::env` 模块的编译错误
  - 验证了应用程序能够成功启动并初始化DNS客户端

- **语言切换功能修复**: 修复了语言切换功能无效的问题
  - 修正了 `ChangeLocale` 消息处理中错误使用旧 `locale` 参数的问题，改为使用新传入的 `locale` 参数
  - 在 `LocaleChanged` 消息处理中添加了配置保存功能，确保语言设置持久化
  - 更新配置中的 `locale` 和 `language` 字段以保持一致性
  - 添加了详细的日志记录和错误处理
  - 验证了语言切换功能正常工作，配置成功保存

### 技术细节
- 文件修改: `src/gui/manager.rs`
- 移除了对已废弃的同步函数 `query_aliyun_dns_list` 的依赖
- 使用现代异步API进行DNS记录管理
- 编译通过，应用程序成功启动