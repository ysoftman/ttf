# ttf

Terminal Tool Finder — 터미널 명령어를 `tools.json`에서 퍼지(fuzzy) 검색하는 Rust CLI 도구.

## 로컬 테스트

```sh
cargo test


# 빌드 없이 실행
cargo run -- <query>
```

## 설치

```sh
cargo build --release
# 바이너리: target/release/ttf
```

## 사용법

```sh
ttf <query>                # 퍼지 검색
ttf -l, --list             # 전체 목록 출력
ttf -d <path>              # tools.json 경로 지정 (기본: exe 디렉토리 또는 ./tools.json)
ttf -n <n>                 # 최대 결과 수 (기본: 20)
```

## 예시

```sh
ttf 디렉토리
 38  ls                list directory contents  [directory, 파일목록, 디렉토리]
 37  tree              display directory structure as a tree  [구조, 트리]
```
