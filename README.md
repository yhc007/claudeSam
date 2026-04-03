# 🦊 claudeSam

Rust AI Agent - Claude Code 스타일의 코딩 어시스턴트

## ✨ 왜 Rust인가?

Python 기반 에이전트 대비 **압도적인 성능 향상**:

| 항목 | Python 에이전트 | 🦀 claudeSam | 개선 |
|------|----------------|--------------|------|
| **시작 시간** | 80-120ms | **~5ms** | **16x 빠름** |
| **메모리 사용** | ~18MB | **~7MB** | **2.6x 절약** |
| **바이너리 크기** | N/A (런타임 필요) | **4.9MB** | 단일 실행파일 |
| **배포** | venv + 의존성 | **복사 1개** | 즉시 배포 |

### 📊 벤치마크 상세

```
=== Startup Time (10회 평균) ===
🦀 Rust claudeSam:  ~5ms (instant)
🐍 Python baseline: ~80-120ms

=== Memory Usage ===
🦀 Rust: 7MB peak
🐍 Python: 18MB+ (anthropic 임포트시)

=== Cold Start ===
🦀 Rust: 첫 실행도 동일 (~5ms)
🐍 Python: 첫 실행 150ms+ (모듈 로딩)
```

### 🎯 실제 의미

- **대화형 모드**: 응답이 즉각적 (Python은 살짝 딜레이)
- **CI/CD 통합**: 빠른 시작으로 파이프라인 최적화
- **서버리스**: Cold start 패널티 최소화
- **임베디드**: 낮은 메모리로 제한된 환경에서도 실행

## 특징

- 🦀 **순수 Rust** - 빠르고 안전한 네이티브 바이너리
- 🔧 **도구 시스템** - Bash, File, Grep 기본 제공
- 💬 **대화형/단발** - chat 모드와 run 모드 지원
- 🔒 **보안** - 위험한 명령 차단
- 🧠 **확장 가능** - pekko-actor, memory-brain 연동 예정

## 설치

```bash
git clone https://github.com/yhc007/claudeSam.git
cd claudeSam
cargo build --release

# 바이너리를 PATH에 추가 (선택)
cp target/release/sam ~/.local/bin/
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
│   ├── main.rs         # CLI 진입점 (clap)
│   ├── api/            # Anthropic API 클라이언트
│   │   └── mod.rs      # Messages API + tool_use
│   ├── engine/         # Agent 루프
│   │   └── mod.rs      # Tool 실행 + 응답 처리
│   ├── tools/          # 도구 구현
│   │   ├── mod.rs      # Tool trait
│   │   ├── bash/       # 명령 실행 (보안 체크)
│   │   ├── file/       # 파일 읽기/쓰기
│   │   └── grep/       # ripgrep 검색
│   ├── config/         # 환경변수 설정
│   ├── memory/         # (예정) 세션 기억
│   └── tui/            # (예정) 터미널 UI
└── Cargo.toml
```

## 핵심 구현

### Tool Trait
```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    async fn execute(&self, input: Value) -> Result<String>;
}
```

### Agent Loop
```rust
loop {
    let response = client.send_message(messages, tools).await?;
    
    if has_tool_use(&response) {
        let results = execute_tools(&response).await;
        messages.push(tool_results);
    } else {
        return response.text;
    }
}
```

## 🗺️ 로드맵

- [x] 기본 CLI (chat, run, tools)
- [x] Anthropic API 클라이언트
- [x] Tool 시스템 (bash, file, grep)
- [ ] pekko-actor 연동 (Actor 기반 병렬 처리)
- [ ] memory-brain 연동 (시맨틱 메모리)
- [ ] TUI 구현 (ratatui)
- [ ] 스트리밍 응답
- [ ] 더 많은 도구 (git, web, etc.)

## 라이선스

MIT

---

*🦊 Sam이 만들었어요!*
