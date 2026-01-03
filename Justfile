build_typescript:
    yarn --cwd=typescript build

run_typescript:
    yarn --cwd=typescript start:medium

build_rust:
    cargo -Z unstable-options -C rust/ build --release

run_rust:
    cargo -Z unstable-options -C rust/ run --release

clean_kernel_cache:
    sudo sysctl -w vm.drop_caches=3

view_perf:
    pprof -http=:8080 rust/profile.pb
