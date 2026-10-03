# 发布 Windows 安装包

工作流：`.github/workflows/release.yml`，使用 GitHub 托管的 Windows 2022 x64 runner。

## 手动发布

在 GitHub 仓库的 **Actions → Windows release → Run workflow** 选择 `main` 并运行。构建成功后会直接发布 GitHub Release，包含 NSIS `.exe`、MSI `.msi` 和 `SHA256SUMS.txt`。版本取自项目配置，例如 `1.2.0` 对应标签 `v1.2.0`；标签不存在时会创建在本次实际构建的提交上。

如果同名标签已指向其他提交，流程会拒绝发布，避免安装包与标签源码不一致。每次新版本发布前应更新项目版本。构建产物也会保留在该次运行的 `windows-x64-installers` Artifact 中。

## 通过版本标签发布

1. 保持 `MuToolsCode/package.json`、`MuToolsCode/src-tauri/Cargo.toml` 和 `MuToolsCode/src-tauri/tauri.conf.json` 的版本一致，并通过 npm/Cargo 正常更新、提交相关锁文件。
2. 在 Windows 上验证构建产物的安装及主要功能。
3. 在需要发布的提交上创建并推送相同版本标签，例如当前版本：

   ```bash
   git tag v1.2.0
   git push origin v1.2.0
   ```

标签推送触发构建；两个安装包均生成后，自动创建公开可见的 GitHub Release（可见范围遵循仓库权限），上传安装包和 SHA-256 校验文件，并生成发行说明。`v1.3.0-beta.1` 这类标签会标记为预发布，项目版本也应同步为 `1.3.0-beta.1`。

发布使用 GitHub 自动提供的 `GITHUB_TOKEN`，无需配置个人 Token；仅发布作业申请 `contents: write`。仓库或组织需允许 GitHub Actions 运行以及该令牌写入 Releases。

工作流不会覆盖已有 Release。若发布步骤失败，先检查是否已经创建 Release：不存在时可重跑失败作业；已存在时应检查并补齐附件，避免重复创建。手动运行的构建产物保留 14 天。

## 范围

- 构建 Windows x64 安装包，不生成仓库辅助工具制作的自解压便携版。
- 未配置 Windows 代码签名证书，产物为未签名安装包。
- 7zip、aria2 等资源包仍按应用现有资源包机制管理。
- 此流程不更新 `MuToosUpdate/update_info.json`；应用内更新源与 GitHub Release 是独立机制。
- Linux 云环境只能验证工作流静态配置和前端构建；Windows 编译、安装与桌面功能以 Actions 运行及 Windows 实测为准。
