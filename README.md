# ttf

Terminal Tool Finder — 터미널 명령어를 `tools.json`에서 퍼지(fuzzy) 검색하는 Rust CLI 도구.

## local test

```sh
cargo test


# 빌드 없이 실행
cargo run -- <query>
```

## install

```sh
cargo build --release
# 바이너리: target/release/ttf
```

## usage

```sh
ttf <query>                # 퍼지 검색
ttf -l, --list             # 전체 목록 출력
ttf -d <path>              # tools.json 경로 지정 (기본: exe 디렉토리 또는 ./tools.json)
ttf -n <n>                 # 최대 결과 수 (기본: 20)
ttf --nocolor              # 색상 출력 끄기 (기본: 색상 on, 커맨드명/태그 색상 표시)
```

## example

```sh
ttf 디렉토리
 38  ls                list directory contents  [directory, 파일목록, 디렉토리]
 37  tree              display directory structure as a tree  [구조, 트리]
```
