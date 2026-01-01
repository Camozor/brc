build_typescript:
		yarn --cwd=typescript build

run_typescript:
		yarn --cwd=typescript start:medium

build_rust:
		cargo -Z unstable-options -C rust/ build --release

run_rust:
		./rust/target/release/brc

view_perf:
		pprof -http=:8080 profile.pb
