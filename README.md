# Tauri App 開發設置

## 1. pnpm 安裝

### 網站 <https://pnpm.io/installation#prerequisites>

#### 1. 在終端機

```bash
winget install -e --id pnpm.pnpm
```

## 2. rust 安裝 ( 內自動安裝 cargo )

### 網站 <https://www.rust-lang.org/zh-TW/tools/install>

### 安裝步驟

#### 1. 網站上下載 64bit 版本並執行

#### 2. 在終端機

1. 選擇 1 (Visual Studio)
2. 等待下載
3. 選擇 1 (推薦安裝)

## 3. tauri 專案創建 ( pnpm & rust 安裝完後 )

### 網站 <https://v1.tauri.app/v1/guides/getting-started/setup/>

### 創建步驟 ( 如果是從github上clone或下載 直接從步驟 3. 執行 )

#### 1. 在終端機

```bash
cd [專案資料夾所在位址]
```

```bash
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
```

```bash
pnpm install
```

```bash
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

將根目錄的 vscode-launch.json 改名成 launch.json 後移至 .vscode 資料夾下便可按 `F5` 啟動偵錯  

## 必讀

前端語言 ( TypeScript & React ) 程式在 /root/src 下 入口由 main.tsx(盡量勿改) 呼叫 App.tsx  
前端 css 由 tailwinds 取代  

後端語言( Rust )程式在 /root/src-tauri/src 下 入口由 main.rs(盡量勿改) 呼叫 lib.rs  
後端 /root/src-tauri 下 ChangeFolder.bat 用以在專案資料夾移動位置時重新建置  
.rs 中多數使用 tokio 做 async(非同步) 開發  

若要 build 成 .exe 或 .msi 在終端機執行  

```bash
cd [專案資料夾]/src-tauri/
```

```bash
pnpm run tauri build
```

使用 ChatGPT  

```bash
tauri開發 使用TypeScript&React&Rust
...
```

*****

## 原始README.md

### Tauri + Vanilla TS

This template should help get you started developing with Tauri in vanilla HTML, CSS and Typescript.

### Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
