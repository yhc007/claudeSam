# 🦊 claudeSam

Rust AI Agent - Claude Code CLI 백엔드

## ✨ 특징

- 🦀 **순수 Rust** - 빠르고 안전한 네이티브 바이너리
- 💳 **API 키 불필요** - Claude Max 플랜으로 바로 사용!
- ⚡ **초고속** - Python 대비 16x 빠른 시작
- 🔧 **도구 시스템** - Bash, File, Grep 기본 제공

## 📊 vs Python 에이전트

| 항목 | Python | 🦀 claudeSam | 개선 |
|------|--------|--------------|------|
| 시작 시간 | 80-120ms | **~5ms** | **16x 빠름** |
| 메모리 | ~18MB | **~7MB** | **2.6x 절약** |
| 바이너리 | 런타임 필요 | **4.9MB** | 단일 파일 |
| API 키 | 필요 | **불필요** | Max 플랜 활용 |

## 설치

```bash
# 사전 요구사항: Claude Code CLI (Max 플랜)
claude --version

# 설치
git clone https://github.com/yhc007/claudeSam.git
cd claudeSam
cargo build --release

# PATH에 추가 (선택)
cp target/release/sam ~/.local/bin/
```

## 사용법

```bash
# 단발 실행 (추천)
sam run "현재 디렉토리의 파일 목록 보여줘"
sam run "이 Rust 코드의 버그 찾아줘: ..."

# 대화형 모드
sam chat

# 도구 목록
sam tools
```

## 아키텍처

```
┌─────────────────────────────────────────────┐
│                 claudeSam                    │
├─────────────────────────────────────────────┤
│  CLI (clap)                                  │
│    ├── sam chat     (대화형)                 │
│    ├── sam run      (단발 실행)              │
│    └── sam tools    (도구 목록)              │
├─────────────────────────────────────────────┤
│  Engine (agent loop)                         │
│    └── ClaudeClient (Claude Code CLI 백엔드) │
├─────────────────────────────────────────────┤
│  Tools                                       │
│    ├── bash   (명령 실행, 보안 체크)         │
│    ├── file   (읽기/쓰기)                    │
│    └── grep   (ripgrep 검색)                 │
└─────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────┐
│  Claude Code CLI (Max 플랜)                  │
│  claude --print --dangerously-skip-perms    │
└─────────────────────────────────────────────┘
```

## 프로젝트 구조

```
claudeSam/
├── src/
│   ├── main.rs         # CLI 진입점
│   ├── api/            # Claude Code CLI 클라이언트
│   ├── engine/         # Agent 루프
│   ├── tools/          # bash, file, grep
│   ├── config/         # 설정
│   ├── memory/         # (예정) 세션 기억
│   └── tui/            # (예정) 터미널 UI
└── Cargo.toml
```

## 🗺️ 로드맵

- [x] 기본 CLI (chat, run, tools)
- [x] Claude Code CLI 백엔드 (API 키 불필요!)
- [x] Tool 시스템 (bash, file, grep)
- [ ] memory-brain 연동 (시맨틱 메모리)
- [ ] pekko-actor 연동 (Actor 병렬 처리)
- [ ] TUI 구현 (ratatui)
- [ ] 스트리밍 응답

## Sam(🦊)의 Sub-Agent로 활용

Clawdbot의 Sam이 복잡한 코딩 작업을 claudeSam에게 위임:

```bash
# Sam이 내부적으로 호출
sam run "pekko-actor에 새 메시지 타입 추가해줘"
```

## 라이선스

MIT

---

*🦊 Sam이 만들었어요!*
