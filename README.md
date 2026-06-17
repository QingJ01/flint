# Flint ⚡ — 一击点燃你的开发环境

> AI 时代的开发环境启动器。从零到能用 AI 写代码，只需一步。

Flint 是一款桌面端工具，面向 Vibecoder 和开发者：可视化一键完成运行时安装、AI 编程工具部署、开发环境初始化，把数小时的环境搭建压缩到几分钟。

## 状态

🚧 **早期开发中** —— 当前处于 Slice 0（行走骨架）阶段。尚不可用。

## 这是什么

- **面向 AI 编程时代** —— 内置 Claude Code、OpenCode、Codex CLI、Cursor 等原生一键安装
- **现代运行时全覆盖** —— Node.js、Bun、Python 多版本管理 + GitHub CLI 开箱即用
- **中国开发者友好** —— npm/pip/GitHub 镜像一键切换，代理一步配置
- **装完就能用** —— 自动处理 PATH、验证可用，不留"装了但跑不起来"的坑
- **全程零提权** —— 仅 WSL 启用一步需管理员，其余全部用户级安装

## 文档

- [产品文档](./Flint-产品文档.md) —— 定位、功能、路线图、商业模式
- [技术设计](./docs/plans/2026-06-17-flint-design.md) —— 架构、核心模块、提权模型
- [Slice 0 实施计划](./docs/plans/2026-06-17-flint-slice-0-plan.md) —— 当前行走骨架任务分解

## 技术栈

Tauri v2 · React + TypeScript · Tailwind CSS · Rust

## 路线图

- **Slice 0** — 行走骨架（Node.js 检测→安装→日志→验证，Windows）
- **Slice 1** — 检测仪表盘完整化
- **Slice 2** — 安装中心横向铺开（全部运行时 + AI 工具）
- **Slice 3** — WSL 全流程 + Preset

## License

待定（将采用开源协议）。
