# 🦊 claudeSam

Rust AI Agent with KAIROS - Claude Code 스타일 + 자율 에이전트

## ✨ 특징

- 🦀 **순수 Rust** - 빠르고 안전한 네이티브 바이너리
- 💳 **API 키 불필요** - Claude Max 플랜으로 바로 사용!
- 🤖 **KAIROS** - 백그라운드 자율 에이전트 모드
- 🌙 **Auto Dream** - 자동 메모리 정리
- ⚡ **16x 빠름** - Python 대비 시작 시간

## 📊 vs Python 에이전트

| 항목 | Python | 🦀 claudeSam | 개선 |
|------|--------|--------------|------|
| 시작 시간 | 80-120ms | **~5ms** | **16x 빠름** |
| 메모리 | ~18MB | **~7MB** | **2.6x 절약** |
| 바이너리 | 런타임 필요 | **4.9MB** | 단일 파일 |

## 설치

```bash
git clone https://github.com/yhc007/claudeSam.git
cd claudeSam
cargo build --release
cp target/release/sam ~/.local/bin/
```

## 사용법

### 기본 명령어
```bash
sam "질문"              # 바로 실행
sam chat                # 대화형 모드
sam tools               # 도구 목록
```

### 🤖 KAIROS 모드
```bash
sam kairos start        # 백그라운드 데몬 시작
sam kairos stop         # 데몬 중지
sam kairos status       # 상태 확인
sam kairos dream        # 수동 메모리 정리
sam kairos log          # 일별 로그 확인
sam kairos log -d 7     # 최근 7일 로그
```

## 🤖 KAIROS 아키텍처

```
┌─────────────────────────────────────────────┐
│                 KAIROS Daemon                │
├─────────────────────────────────────────────┤
│  Auto Dream (자동 메모리 정리)               │
│    ├── Time Gate: 24시간 경과                │
│    ├── Session Gate: 5+ 세션                 │
│    └── Lock Gate: 분산 락                    │
├─────────────────────────────────────────────┤
│  Memory System                               │
│    ├── MEMORY.md (인덱스, 200줄 제한)        │
│    ├── daily_log/ (일별 로그)                │
│    └── consolidation lock (PID 기반)         │
└─────────────────────────────────────────────┘
```

## 프로젝트 구조

```
claudeSam/
├── src/
│   ├── main.rs           # CLI + KAIROS 명령어
│   ├── api/              # Claude Code CLI 클라이언트
│   ├── engine/           # Agent 루프
│   ├── tools/            # bash, file, grep
│   ├── config/           # 설정
│   └── kairos/           # 🤖 KAIROS 모듈
│       ├── mod.rs        # 메인
│       ├── daemon.rs     # 백그라운드 데몬
│       ├── auto_dream.rs # 자동 메모리 정리
│       ├── consolidation.rs # 분산 락
│       ├── memdir.rs     # 메모리 디렉토리
│       └── daily_log.rs  # 일별 로그
└── Cargo.toml
```

## 🗺️ 로드맵

- [x] 기본 CLI
- [x] Claude Code CLI 백엔드
- [x] Tool 시스템
- [x] **KAIROS 데몬**
- [x] **Auto Dream (자동 메모리 정리)**
- [x] **일별 로그 시스템**
- [ ] GitHub Webhook 연동
- [ ] 푸시 알림
- [ ] memory-brain 연동
- [ ] pekko-actor 연동

## 라이선스

MIT

---

*🦊 Sam + 🤖 KAIROS*
