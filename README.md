# ttf

Terminal Tool Finder — 터미널 명령어를 `tools.json`에서 퍼지(fuzzy) 검색하는 CLI 도구.

## install and usage

```bash
# install (crates.io)
cargo uninstall terminal-tool-finder; cargo install terminal-tool-finder

# 재설치 (이미 설치된 경우)
cargo install terminal-tool-finder --force

# install (로컬 소스에서 빌드)
cargo build --release
# 바이너리: target/release/ttf

# help
ttf -h

# 퍼지 검색
ttf <query>

# 전체 목록 출력
ttf -l, --list

# tools.json 경로 지정 (기본: exe 디렉토리 또는 ./tools.json)
ttf -d <path>

# 최대 결과 수 (기본: 20)
ttf -n <n>

# 색상 출력 끄기 (기본: 색상 on, 커맨드명/태그 색상 표시)
ttf --nocolor
```

## build and deploy

```bash
# local test
cargo test

# 빌드 없이 실행
cargo run -- <query>

# cargo 로그인
# <https://crates.io/me> 에서 토큰 생성함
# 로그인 하면 ~/.cargo/credentials.toml 에 토큰 저장됨
cargo login

# cargo.toml 버전업 수정 -> git 커밋 -> cargo 로 배포
# --allow-dirty : git 커밋 없이 로컬 변경 사항이 있는채로 배포 허용
cargo publish
```
