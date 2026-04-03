# 🦊 claudeSam

Rust AI Agent - Claude Code 스타일의 코딩 어시스턴트

## 특징

- 🦀 **순수 Rust** - 빠르고 안전한 네이티브 바이너리
- 🔧 **도구 시스템** - Bash, File, Grep 기본 제공
- 💬 **대화형/단발** - chat 모드와 run 모드 지원
- 🔒 **보안** - 위험한 명령 차단

## 설치

```bash
git clone https://github.com/yhc007/claudeSam.git
cd claudeSam
cargo build --release
```

## 사용법

```bash
# API 키 설정
export ANTHROPIC_API_KEY="your-key"

# 대화형 모드
sam chat

# 단발 실행
sam run "현재 디렉토리의 Rust 파일 목록 보여줘"

# 도구 목록
sam tools
```

## 프로젝트 구조

```
claudeSam/
├── src/
│   ├── main.rs         # CLI 진입점
│   ├── api/            # Anthropic API 클라이언트
│   ├── engine/         # Agent 루프 (도구 실행)
│   ├── tools/          # 도구 구현
│   │   ├── bash/       # 명령 실행
│   │   ├── file/       # 파일 읽기/쓰기
│   │   └── grep/       # 패턴 검색
│   ├── config/         # 설정
│   ├── memory/         # (예정) 세션 기억
│   └── tui/            # (예정) 터미널 UI
└── Cargo.toml
```

## 로드맵

- [ ] pekko-actor 연동 (Actor 기반 병렬 처리)
- [ ] memory-brain 연동 (시맨틱 메모리)
- [ ] TUI 구현 (ratatui)
- [ ] 스트리밍 응답
- [ ] 더 많은 도구 (git, web, etc.)

## 라이선스

MIT
