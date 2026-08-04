# 月序 / Lunar Almanac

<p align="center">
  <img src="assets/yuexu-icon.png" width="96" alt="月序图标">
</p>

> 一张每天自动更新的全年农历桌面日历。更新时由 Rust 进程直接生成静态 PNG，完成后立即退出，没有常驻服务、浏览器、WebView 或网络连接。

![深色主题预览](assets/preview-dark.png)

## 产品状态

- Windows 10/11 x64
- 全年 12 个月、农历、闰月、当天高亮
- 深色与浅色主题，默认深色，主题会被本地记住
- 登录和每天 `00:01` 自动更新；错过零点会在系统恢复后补跑
- 用户级安装，不要求管理员权限
- 常规更新使用 Rust + `lunar_rust` + `resvg` + `tiny-skia`，不依赖 Chrome、Edge 或 WebView

日历的手动预览仍会按用户默认浏览器打开一个本地 HTML 设置页，只有用户主动点开快捷方式时才会发生；计划任务和每日壁纸更新不会启动浏览器。

## 安装

从 Release 下载 `YueXu-<版本>-windows-x64.zip`，解压后在 PowerShell 运行：

```powershell
Set-ExecutionPolicy -Scope Process Bypass
.\Install-YueXu.ps1
```

安装完成后，桌面和开始菜单会出现“月序日历”。打开它可以预览全年日历并切换深浅主题；点击主题色块会立即应用到桌面，之后的自动更新沿用同一主题。

## 命令行

```powershell
# 立即更新，使用记住的主题
.\LunarCalendar.exe --update --quiet

# 切换并记住浅色主题
.\LunarCalendar.exe --update --theme light

# 生成图片但不设置为壁纸
.\LunarCalendar.exe --update --width 1920 --height 1080 --set-wallpaper=false

# 打开预览与设置
.\LunarCalendar.exe --preview
```

生成的壁纸与偏好只写入 `%LOCALAPPDATA%\YueXu`。不读取日历账户、不上传数据、不需要 API Key。

## 架构

| 层 | 实现 |
| --- | --- |
| 日历与农历 | Rust，`lunar_rust` 离线历法 |
| 版式与 PNG | Rust 生成 SVG，`resvg` / `tiny-skia` 进程内栅格化 |
| Windows 集成 | 原生 Win32 `SystemParametersInfoW` 设置壁纸、Windows 任务计划程序定时触发 |
| 预览设置 | 仅按需打开的本地 HTML 页面 |

## 面向商业发行

这个仓库提供可复现的 Windows 发行构建，而不是只提供开发脚本：

```powershell
.\scripts\Build-Release.ps1
```

构建会生成 GUI 子系统的 `LunarCalendar.exe`、安装器、卸载器、ICO 图标、SHA256 校验和和版本化 zip。发布与定价策略见 [docs/商业化方案.md](docs/商业化方案.md)，正式发布前检查 [docs/发布清单.md](docs/发布清单.md)。

## 开发

```powershell
cargo test --locked
cargo run -- --update --width 1920 --height 1080 --set-wallpaper=false
```

Rust 运行时代码在 `native/`。`index.html` 和 `src/` 仅用于用户手动打开的预览页，不参与壁纸生成。

## 开源

月序以 MIT License 开源。开源版可以自由使用、修改和分发；商业发行可提供签名安装包、专属主题包、批量部署和优先支持。贡献方式见 [CONTRIBUTING.md](CONTRIBUTING.md)。
