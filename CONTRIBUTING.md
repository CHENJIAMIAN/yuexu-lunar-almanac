# 贡献指南

> [English](CONTRIBUTING.en.md)

欢迎提交 Issue、设计建议和 Pull Request。

## 本地验证

```powershell
cargo test --locked
cargo run -- --update --width 1920 --height 1080 --set-wallpaper=false
```

涉及视觉改动时，请分别检查深色与浅色主题，以及 1920×1080 和 3840×2160 输出。涉及农历逻辑时，请覆盖春节、闰月和跨公历年的日期。壁纸渲染必须保持浏览器无依赖。

## 提交约定

- 提交信息和用户可见文案使用中文。
- 不引入外部网络请求、遥测或广告 SDK。
- 不提交 `output/`、`dist/`、`release/` 下的构建产物。
