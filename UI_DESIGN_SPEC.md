# UI_DESIGN_SPEC

> SnapScribe UI 唯一视觉事实源。来源：《本地视频录音转文字工具_开发文档》§12–§36，新增历史管理与渐进式转录展示规范。
> 任何 UI 改动不得引入本文之外的颜色、圆角、阴影、字体、图标库与页面导航。

---

## 1. 总体风格

```text
现代极简 · 轻量 · 柔和 · 精致 · 低干扰 · 桌面工具感
```

不要设计成：传统 Windows 工具 / SaaS Dashboard / AI 科技风后台 / 音乐播放器。

单窗口结构：

```text
┌────────────────────────────────────────────┐
│ App Header                                 │
├────────────────────────────────────────────┤
│  Upload / File Summary                     │
│  Progress / Player                         │
│  Transcript Result（渐进式追加）           │
├────────────────────────────────────────────┤
│ Bottom Action Bar                          │
└────────────────────────────────────────────┘
```

禁止：复杂 Sidebar、多栏后台、Dashboard、多页面导航。

## 2. Colors

```css
:root {
  --color-primary: #4353FF;
  --color-primary-soft: #EEF0FF;

  --color-accent: #FF6B6B;
  --color-accent-soft: #FFAAA5;

  --color-bg: #F7F8FC;
  --color-surface: #FFFFFF;
  --color-surface-soft: #EEF0FF;

  --color-text-primary: #111111;
  --color-text-secondary: #6F7785;
  --color-text-tertiary: #9AA3B2;

  --color-border: #E5E5EA;
  --color-divider: #F0F0F5;

  --color-success: #22A06B;
  --color-warning: #E59B18;
  --color-error: #D94F4F;
}
```

用法：`#4353FF` 主按钮/主进度/时间戳；`#EEF0FF` 弱背景/选中态；`#FF6B6B` 风险删除错误；`#111111` 主文本；`#9AA3B2` 辅助文字。禁止自增品牌色。

## 3. 圆角

```css
--radius-sm: 8px;   /* 小标签 */
--radius-md: 12px;  /* 输入框 / Button */
--radius-lg: 16px;  /* 文件卡片 / 播放器 / DropZone */
--radius-xl: 24px;  /* 大容器 */
--radius-round: 999px; /* 圆形按钮 */
```

禁止自定义 10 / 14 / 18 / 22px（Modal 的 18px 为规范明确允许值）。

## 4. 间距

以 4px 为基准：`4 / 8 / 12 / 16 / 20 / 24 / 32 / 40 / 48`。
图标↔文字 8/12，控件内 12/16，卡片内 20/24，模块间 24/32，页边距 32/40。禁止 13/19/27 类无规律值。

## 5. 字体与字号

```css
font-family: "SF Pro Display", Inter, -apple-system, BlinkMacSystemFont,
  "Segoe UI", "Microsoft YaHei", Arial, sans-serif;
/* 数字/时间 */ font-family: "SF Mono", Consolas, monospace;
```

| 场景 | 字号 | 字重 |
|---|---:|---:|
| 页面标题 | 24 | 700 |
| 区块标题 | 18 | 600 |
| 文件名 | 14 | 600 |
| 正文 | 14 | 400 |
| Button | 14 | 600 |
| 时间戳 | 12 | 500 |
| 辅助信息 | 12 | 400 |

禁止营销型大标题。

## 6. 阴影

```css
--shadow-sm: 0 2px 12px rgba(0,0,0,0.06);
--shadow-md: 0 12px 36px rgba(17,17,17,0.10);
--shadow-primary: 0 16px 36px rgba(67,83,255,0.20);
```

普通卡片无阴影；Hover/Active 可 `shadow-sm`；Modal `shadow-md`；主色强阴影仅极少数焦点。禁止发光、重阴影、多层阴影。

## 7. Header

高 56–64px；左：产品名 SnapScribe；右：「历史」入口 + 设置占位。样式 `background:#FFFFFF; border-bottom:1px solid #F0F0F5;` 默认无阴影。

## 8. DropZone

默认：白底、`2px dashed #C9CBE0`、圆角 16、padding 32；中心 48×48 圆形 `#4353FF` 白色上传 icon；文案「拖入视频或录音 / 或点击选择文件」+ 格式行 `MP4 · MOV · MKV · MP3 · M4A · WAV`。
Hover：边框 `#4353FF`、背景 `#F8F9FF`；Drag Over：背景 `#EEF0FF` + `scale(1.01)` + `transition:150ms ease`。必须同时支持点击选择。

## 9. FileItem

`[图标38×38 r10 bg#EEF0FF] 名称(13–14px/600) + 辅助信息(12px/#9AA3B2) [×]`；白底、`1px #EEF0FF`、圆角 12、padding 12×14。

## 10. Button

- Primary：bg `#4353FF`、白字、r12、h40、padding 0×20、14px/600；同视图仅一个最强 Primary。
- Secondary：白底、字 `#111111`、`1px solid #E5E5EA`。
- Danger：`#FF6B6B`，只用于删除类不可逆操作。
- Disabled：`opacity:.45; cursor:not-allowed`；Loading 态 Spinner + 文案且宽度稳定。

## 11. 转写进度（含分段阶段）

```text
正在转写 meeting.mp4                    38%
[波形条 28–32px：未处理 #C9CBE0 / 已处理 #4353FF]
██████████████░░░░░░░░░
已处理 08:12 / 21:35 · 第 8 / 22 段
```

Progress bar：h4、track `#E5E5EA`、fill `#4353FF`。波形只是辅助信息。preparing/splitting 阶段显示对应文案（「正在提取音频并切片…」），波形未点亮。

## 12. Audio Player（P1）

`[▶] 00:32 ━━━━━━━━━━━━━━━ 12:48` 高 48–56、白底、`1px #E5E5EA`、r16；主播放按钮 `#4353FF` 圆形，其余白底描边。定位为转写结果的辅助导航。

## 13. Transcript（渐进式渲染）

```text
00:03   今天我们讨论一下这个项目。
00:12   第一部分是当前开发进度。
```

正文 14px `#111111`；时间戳 12px `#4353FF` 等宽字体、固定宽 56–64px；行高 1.7；item padding 12–16；分割线 `#F0F0F5`；Hover 背景 `#F8F9FF`；当前播放段背景 `#EEF0FF` + r12；点击时间戳跳转播放并激活该段。

**渐进式追加规范**：转写中新段落从列表尾部插入，自动滚动到底部（用户手动上滚时暂停自动滚动）；新段落不做弹跳/长动画，仅允许 150ms ease 淡入；列表顶部显示已接收段数计数（辅助信息样式）。等待中的下一段显示一行占位提示「正在识别第 N 段…」（tertiary 色，不闪烁）。

## 14. Empty / Error State

Empty：一个图标 + 「暂无转写内容」+ 一句说明 + 一个主操作；禁止大插画。
Error：图标/标题用 `#D94F4F`，浅红仅局部提示，不整块红底；提供「重新选择」。

## 15. Modal

白底、r18、padding 24、`shadow-md`；确认类 360–420px、设置/历史类 480–560px；按钮右对齐、Primary 最右、gap 12。用于：历史管理、删除确认、导出确认。

## 16. Toast

bg `#111111` 白字、r12、padding 12×16；用于「已复制全文 / TXT 导出成功 / SRT 导出成功 / 已保存到历史」；1800–2400ms；严重错误不用 Toast 单独表达。

## 17. Icon 与动效

只用 Lucide Icons，16/18/20px 统一 stroke-width；禁止 Emoji 作正式图标。
动效统一 `transition:150ms ease`（Hover/Button/Focus/Drag Over/Toggle/Transcript Active）；禁止弹跳、大幅位移、强缩放、装饰循环动画（Spinner/波形除外）。

## 18. 窗口与响应式

默认 1080×720，最小 820×560，内容最大宽度 960–1040px；宽度 <860px 时页边距 24px、双栏→单栏、优先保留 Transcript 阅读面积。

## 19. Accessibility

Focus 可见；全部按钮支持键盘；Icon Button 有 `aria-label`；拖拽必须同时支持点击选择；状态不只依赖颜色；正文字号 ≥12px；保证对比度。

## 20. Do / Don't

Do：大面积留白、稳定圆角体系、主色只用于关键动作、浅灰/薰衣草灰分层、页面只强调一个主任务、Transcript 阅读效率优先。
Don't：渐变背景、玻璃拟态、科技发光、Card 套 Card、重阴影、KPI Dashboard、复杂 Sidebar、多彩标签、多个强主按钮、为装饰降低信息密度。
