# UI-SPEC — DevCave 界面设计规范

> 版本：v0.1 | 日期：2026-08-02

---

## 1. 设计原则

| 原则 | 描述 |
|------|------|
| **极简主义** | 移除所有非必要的装饰元素，让内容成为焦点 |
| **深色优先** | 默认暗色主题，这是开发者最熟悉的视觉环境 |
| **内容密度** | 在不牺牲可读性的前提下，最大化信息密度 |
| **代码感** | 等宽字体、代码块、命令行风格的交互暗示 |
| **精准空白** | 空白是设计元素，而非留白，遵守 8px 基础栅格 |

---

## 2. 色彩系统

### 2.1 背景色（Background Palette）

| 变量名 | HEX | RGB | 用途 |
|--------|-----|-----|------|
| `bg-base` | `#0D0D0D` | 13,13,13 | 最深背景，页面根背景 |
| `bg-surface` | `#141414` | 20,20,20 | 卡片、侧边栏背景 |
| `bg-elevated` | `#1E1E1E` | 30,30,30 | 悬浮元素、输入框 |
| `bg-overlay` | `#2A2A2A` | 42,42,42 | Hover 状态、高亮区域 |
| `bg-border` | `#2E2E2E` | 46,46,46 | 分割线、边框 |

```
深度层次示意：
  页面背景 #0D0D0D
  └─ 侧边栏 #141414
     └─ 卡片 #1E1E1E
        └─ 输入框/代码块 #2A2A2A
```

### 2.2 主色调（Primary Palette）

| 变量名 | HEX | 用途 |
|--------|-----|------|
| `primary-600` | `#7C3AED` | 主要交互元素（按钮、链接） |
| `primary-700` | `#6D28D9` | Hover/Active 状态 |
| `primary-400` | `#A78BFA` | 辅助强调、徽章 |
| `primary-900` | `#2E1065` | 极低饱和背景（选中态背景） |

**主色使用原则**：
- CTA 按钮（Call-to-Action）使用 `primary-600`
- 文字链接使用 `primary-400`（保证暗色背景可读性）
- 不用于大面积背景，避免视觉疲劳

### 2.3 文字色（Text Palette）

| 变量名 | HEX | 用途 |
|--------|-----|------|
| `text-primary` | `#F5F5F5` | 主要正文、标题 |
| `text-secondary` | `#A1A1AA` | 次要信息、时间戳、描述文字 |
| `text-muted` | `#71717A` | 占位符、禁用状态 |
| `text-inverse` | `#0D0D0D` | 亮色按钮上的文字 |

### 2.4 功能色（Functional Colors）

| 变量名 | HEX | 用途 |
|--------|-----|------|
| `success` | `#22C55E` | 成功状态、代码 Diff 新增行 |
| `danger` | `#EF4444` | 错误状态、代码 Diff 删除行 |
| `warning` | `#F59E0B` | 警告、待处理标记 |
| `info` | `#3B82F6` | 信息提示、链接 |

### 2.5 情绪色（Tree House 专用）

| 情绪标签 | HEX | 描述 |
|---------|-----|------|
| 愤怒 😤 | `#EF4444` | 红色 |
| 迷茫 😕 | `#6366F1` | 紫蓝 |
| 焦虑 😰 | `#F59E0B` | 琥珀 |
| 崩溃 💀 | `#71717A` | 灰色 |
| 摸鱼 🐟 | `#10B981` | 青绿 |
| 治愈 ☀️ | `#FCD34D` | 暖黄 |

---

## 3. 字体系统

### 3.1 字体栈

```css
/* 主字体（界面文字） */
font-family: 'Inter', 'PingFang SC', 'Microsoft YaHei', system-ui, sans-serif;

/* 代码字体（代码块、片段、技术标签） */
font-family: 'Geist Mono', 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
```

### 3.2 字体尺寸规范

| 用途 | 大小 | 字重 | 行高 | 说明 |
|------|------|------|------|------|
| 页面大标题 | 24px | 600 | 1.3 | 文章标题、模块标题 |
| 区块小标题 | 18px | 600 | 1.4 | 卡片标题、段落标题 |
| 子标题 | 16px | 500 | 1.5 | 侧边栏项目标题 |
| 正文 | 14px | 400 | 1.6 | 主要阅读文字 |
| 辅助文字 | 13px | 400 | 1.5 | 标签、时间、计数 |
| 微文字 | 12px | 400 | 1.4 | 版权、极次要信息 |
| 代码正文 | 14px | 400 | 1.7 | 代码块内容，等宽 |

### 3.3 中文排版注意事项

- 中英文混排时，中英文之间加 1 个半角空格（推荐 CSS `word-spacing` 控制）
- 代码内嵌于中文段落时，使用 `<code>` 标签，背景色 `bg-overlay`，内边距 2px 4px

---

## 4. 间距与栅格系统

### 4.1 基础单位

**基础单位：8px**

```
4px   ← xs（细节调整）
8px   ← sm（组件内部间距）
12px  ← md（卡片内边距）
16px  ← lg（组件间间距）
24px  ← xl（区块间距）
32px  ← 2xl（页面级间距）
48px  ← 3xl（大段落间距）
64px  ← 4xl（页面边距）
```

### 4.2 布局栅格

| 布局区域 | 宽度 | 说明 |
|---------|------|------|
| 左侧导航栏（展开） | 220px | 固定宽度 |
| 左侧导航栏（折叠） | 60px | 仅图标 |
| 主内容区 | 自适应 | `width: calc(100% - 220px - 280px)` |
| 右侧小组件栏 | 280px | 固定宽度 |
| 内容区最大宽度 | 720px | 单栏内容（文章详情、编辑器） |
| 卡片列宽 | 自适应 3列 | 响应式 grid，gap: 12px |

### 4.3 圆角规范

| 层级 | 值 | 适用 |
|------|-----|------|
| 小 | 4px | 标签、徽章、代码块角落 |
| 中 | 8px | 按钮、输入框、小卡片 |
| 大 | 12px | 主卡片、面板 |
| 超大 | 16px | 模态框、抽屉面板 |
| 圆形 | 50% | 头像、图标按钮 |

---

## 5. 核心组件规范

### 5.1 按钮（Button）

```
┌─────────────────────────────────────────┐
│  变体      │  背景          │  文字      │
├─────────────────────────────────────────┤
│  Primary   │  #7C3AED      │  #F5F5F5  │
│  Secondary │  #2A2A2A      │  #F5F5F5  │
│  Ghost     │  透明          │  #A1A1AA  │
│  Danger    │  #EF4444      │  #F5F5F5  │
│  Disabled  │  #1E1E1E      │  #71717A  │
└─────────────────────────────────────────┘

尺寸规格：
  sm：height 28px, padding 8px 12px, font-size 13px
  md：height 36px, padding 10px 16px, font-size 14px（默认）
  lg：height 44px, padding 12px 24px, font-size 16px

Hover 效果：背景亮度 +10%（brightness(1.1)），0.15s ease
Active 效果：背景亮度 -5%，scale(0.97)，0.1s ease
Focus：2px solid #7C3AED outline，offset 2px
```

### 5.2 输入框（Input）

```
默认态：
  background: #1E1E1E
  border: 1px solid #2E2E2E
  border-radius: 8px
  height: 36px（单行）
  padding: 0 12px
  font-size: 14px
  color: #F5F5F5
  placeholder-color: #71717A

Focus 态：
  border-color: #7C3AED
  box-shadow: 0 0 0 3px rgba(124, 58, 237, 0.15)

Error 态：
  border-color: #EF4444
  box-shadow: 0 0 0 3px rgba(239, 68, 68, 0.15)

Textarea：
  min-height: 120px
  resize: vertical
  line-height: 1.6
```

### 5.3 卡片（Card）

```
基础卡片：
  background: #1E1E1E
  border: 1px solid #2E2E2E
  border-radius: 12px
  padding: 16px

Hover 态（可点击卡片）：
  border-color: #3A3A3A
  background: #222222
  transition: all 0.2s ease

代码预览区域（卡片内）：
  background: #141414
  border-radius: 8px
  padding: 12px
  font-family: Geist Mono
  font-size: 13px
  line-height: 1.7
  max-height: 160px
  overflow: hidden
  使用渐变遮罩表示截断：
    mask-image: linear-gradient(to bottom, black 80%, transparent 100%)
```

### 5.4 标签（Tag）

```
技术栈标签：
  background: #1E1E1E
  border: 1px solid #3A3A3A
  border-radius: 4px
  padding: 2px 8px
  font-family: Geist Mono
  font-size: 12px
  color: #A78BFA（紫色调，强调技术感）

情绪标签（树洞）：
  带情绪对应背景色（15% 透明度）+ 对应文字色
  border-radius: 99px（胶囊形状）
  padding: 4px 10px

互动标签（点赞数/评论数）：
  color: #71717A
  font-size: 13px
  gap: 4px（图标与数字）
  hover: color: #A1A1AA
```

### 5.5 代码块（Code Block）

```
容器：
  background: #0D0D0D（最深背景，与页面区分）
  border: 1px solid #2E2E2E
  border-radius: 8px
  overflow: hidden

语言标识栏（顶部）：
  background: #141414
  padding: 8px 16px
  font-family: Geist Mono
  font-size: 12px
  color: #71717A
  right-side: 一键复制按钮

代码区域：
  padding: 16px
  font-size: 14px
  line-height: 1.7
  tab-size: 2

代码高亮主题：One Dark（参考 VS Code）
  关键字：#C678DD（紫）
  字符串：#98C379（绿）
  数字：#D19A66（橙）
  注释：#5C6370（灰，斜体）
  函数名：#61AFEF（蓝）

Diff 显示规则：
  新增行：background rgba(34, 197, 94, 0.12)，左侧 2px solid #22C55E
  删除行：background rgba(239, 68, 68, 0.12)，左侧 2px solid #EF4444
  行号列：30px 宽，color: #71717A
```

### 5.6 积分/等级徽章

```
样式规格：
  形状：圆角矩形 + 左侧色块
  字体：Geist Mono
  大小：12px

等级对应色：
  Lv.1：#71717A（灰）
  Lv.2：#22C55E（绿）
  Lv.3：#3B82F6（蓝）
  Lv.4：#F59E0B（金）
  Lv.5：#A78BFA（紫）
  Lv.6：渐变 #FFD700 → #FF6B35（荣耀橙金）

进度条：
  height: 3px
  border-radius: 99px
  background: #2A2A2A
  fill: 对应等级色
```

### 5.7 通知/Toast

```
位置：右下角，margin 24px
宽度：320px
border-radius: 12px
padding: 14px 16px
border-left: 3px solid [对应功能色]
background: #1E1E1E

类型与图标：
  success：✓ #22C55E
  error：✗ #EF4444
  info：ℹ #3B82F6
  warning：⚠ #F59E0B

动画：
  入场：slide-in-from-right 0.3s ease
  出场：fade-out 0.2s ease（3秒后自动消失）
```

---

## 6. 动效规范

### 6.1 基础过渡

```css
/* 颜色/背景切换 */
transition: color 0.15s ease, background-color 0.15s ease, border-color 0.15s ease;

/* 尺寸/位置变化 */
transition: transform 0.2s ease, opacity 0.2s ease;

/* 重要交互反馈 */
transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
```

### 6.2 页面级动画

| 场景 | 动画 | 时长 |
|------|------|------|
| 页面切换 | fade + translateY(8px) | 0.25s |
| 卡片 Feed 加载 | stagger fade-in（间隔 50ms）| 0.3s |
| 抽屉打开 | slide-in-from-right | 0.3s cubic-bezier |
| 模态框出现 | scale(0.95→1) + fade-in | 0.2s |
| Toast 通知 | slide-in-from-right | 0.3s |

### 6.3 微交互

| 元素 | 交互 | 动效 |
|------|------|------|
| 点赞按钮 | 点击时 | scale(1.2) 弹跳，0.15s |
| 卡片 Hover | 鼠标移入 | border 亮度提升，0.2s |
| 导航项 Hover | 鼠标移入 | 左侧指示线展开（width 0→3px），0.15s |
| 积分增加 | 获得积分时 | 数字滚动动画，0.5s |
| 代码复制 | 复制成功 | 图标变✓，1.5s 后还原 |

---

## 7. 图标系统

使用 **Lucide Icons**（与 React 生态高度兼容，风格极简线性）

| 模块 | 图标 |
|------|------|
| 发现 | `Compass` |
| 技术分享 | `FileCode` |
| 协作匹配 | `Users` |
| 匿名树洞 | `TreePine` |
| 技术问答 | `MessageCircleQuestion` |
| 技术活动 | `CalendarDays` |
| 设置 | `Settings` |
| 点赞 | `Heart` |
| 评论 | `MessageSquare` |
| 收藏 | `Bookmark` |
| 分享 | `Share2` |
| 代码 | `Code2` |
| 积分 | `Zap` |

**规格**：默认 16px（行内），20px（导航），24px（强调图标）

---

## 8. 响应式断点

| 断点 | 值 | 布局变化 |
|------|-----|---------|
| `xs` | < 480px | 单列，隐藏左侧导航，底部 Tab 栏 |
| `sm` | 480-768px | 单列，底部 Tab 栏 |
| `md` | 768-1024px | 两列（无右侧组件栏），左侧导航折叠 |
| `lg` | 1024-1280px | 三列完整布局，左侧导航展开 |
| `xl` | > 1280px | 同 lg，内容区有内边距，最大宽度限制 |

---

## 9. 可访问性规范

| 要求 | 实现方式 |
|------|---------|
| 颜色对比度 | 正文 `#F5F5F5` 在 `#141414` 背景上：对比度 17.9:1（AAA） |
| 焦点可见性 | 所有交互元素有 2px 紫色 outline，`focus-visible` 触发 |
| 键盘导航 | Tab 键遍历所有交互元素，Escape 关闭 Overlay |
| ARIA 标签 | 图标按钮必须有 `aria-label` |
| 语义化 HTML | 使用 `<nav>`、`<main>`、`<article>`、`<aside>` 等语义标签 |

---

*UI-SPEC 版本历史*

| 版本 | 日期 | 变更 |
|------|------|------|
| v0.1 | 2026-08-02 | 初稿，基于 DevCave 设计原则制定 |
