# HuanYu Tauri App

## pnpm 安裝

### 網站 <https://pnpm.io/installation#prerequisites>

#### 1. 在 windows powershell

```bash
winget install -e --id pnpm.pnpm
```

## rust 安裝 ( 內自動安裝 cargo )

### 網站 <https://www.rust-lang.org/zh-TW/tools/install>

### 安裝步驟

#### 1. 網站上下載 64bit 版本並執行

#### 2. 在終端機

1. 選擇 1 (Visual Studio)
2. 等待下載
3. 選擇 1 (推薦安裝)

## tauri 專案創建

### 網站 <https://v1.tauri.app/v1/guides/getting-started/setup/>

### 創建步驟 ( 如果是從github上clone或下載 直接由3.執行 )

#### 1. 在終端機

```bash
cd [專案資料夾所在位址]
pnpm create tauri-app --tauri-version 1
```

#### 2. 選擇  

Project name -> [專案名稱]  
Identifier -> [專案id]  
Choose which language to use for your frontend -> TypeScript / JavaScript - (pnpm, yarn, npm, deno, bun)  
Choose your package manager -> pnpm  
Choose your UI template -> Vanilla  
Choose your UI flavor -> TypeScript  

#### 3. 在終端機

```bash
cd [專案資料夾]
pnpm install
pnpm tauri dev
```

## VSCode 延伸模組

### 必裝

Tauri  
Rust (auther: 1yic)  
Rust Doc Viewer  
Dependi  

### ( 可選 )

JavaScript and TypeScript Nightly  
Simple React Snippets  
ES7+ React/Redux/React-Native snippets  
Prettier - Code formatter  
Pretty TypeScript Errors  

## VSCode 偵錯

將根目錄的 vscode-launch.json 改名成 launch.json 後移至 .vscode 資料夾下便可 `F5` 啟動偵錯

*****

## 以下可忽略 README.md

### Tauri + Vanilla TS

This template should help get you started developing with Tauri in vanilla HTML, CSS and Typescript.

### Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
