build:
    cd both && yarn build

run:
    node both/src/main.js

clean_kernel_cache:
    sudo sysctl -w vm.drop_caches=3

view_perf:
    pprof -http=:8080 rust/profile.pb
