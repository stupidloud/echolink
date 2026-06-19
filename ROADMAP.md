# EchoLink Roadmap

## 接入本地 ONNX ASR(离线识别)

**目标**:在现有的云端 ASR 协议(StepFun SSE / OpenAI 兼容 / OpenRouter)之外,新增一个**纯本地、离线**的识别协议,无 API 费用、隐私好、断网可用。

### 选型

- **引擎**:[sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx)(k2-fsa / Next-gen Kaldi),纯本地、运行时只依赖 ONNX Runtime,跨平台,Apache-2.0。
- **Rust 绑定**:`sherpa-rs`,封装了 fbank 特征提取 + 解码 + 分词,后端只需喂 16k PCM → 拿回文本。
- **模型**:**SenseVoice**(阿里,zh/en/ja/ko/粤,非流式,CPU 快),正好匹配"按住说完再上屏"的交互,中文为主定位。
  - 官方 sherpa-onnx 打包:`sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17`
  - 只需 2 个文件:`model.int8.onnx`(~228MB)+ `tokens.txt`

### 有利前提

- 现有 StepFun(SSE)路径的 `startPcmRecording` + `floatTo16BitPCM` 已经采集 **16kHz / mono / 16-bit PCM**,正是 ONNX 模型要的输入——**前端音频侧几乎不用动**,`local` 协议复用 PCM 采集即可。
- 已有的 `normalize_script`(简繁转换)可直接复用。

### 模型下载(已验证直链可下,无需任何 CLI / token)

单文件直下,**ModelScope/HF 路线连解压都省了**(不需要 tar/bzip2 依赖):

```
# 国内默认:ModelScope 镜像(社区镜像,建议自建 fork 保稳定)
https://www.modelscope.cn/models/Mr7Cat/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17/resolve/master/model.int8.onnx
https://www.modelscope.cn/models/Mr7Cat/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17/resolve/master/tokens.txt

# 海外/备用:HuggingFace 单文件
https://huggingface.co/csukuangfj/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17/resolve/main/model.int8.onnx
https://huggingface.co/csukuangfj/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17/resolve/main/tokens.txt

# 官方整包(需 bzip2 解压,含 fp32+int8,~1GB)
https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17.tar.bz2
```

- 直链规律:`https://www.modelscope.cn/models/<ns>/<model>/resolve/master/<file>`
- 实测全部支持 Range(HTTP 206)→ **可断点续传**。

### 分步计划(风险逐级释放)

1. **手填路径 + 云端编译验证**:加 `local` 协议 + `transcribe_audio_local`,模型路径在设置里手填(零网络代码),先把 sherpa-rs 在云端编过、**跑通一句中文**。← 真正的风险点,先打通这一步。
2. **一键下载**:加下载命令,默认 ModelScope 源 + 进度条 + 源切换。
3. **可选增强**:sha256 校验、断点续传、热词(参考 `dengcunqin/SenseVoiceSmall_hotword`,FunASR 格式)。
