# Builds `deli` and `deli-derive`
build:
    @echo 'Building...'
    cargo build

alias test := test-chrome

# Runs browser tests for `deli` using chrome
test-chrome:
    @echo 'Testing...'
    cd deli && wasm-pack test --chrome

# Runs browser tests for `deli` using chrome (intended for use in CI)
test-chrome-headless:
    @echo 'Testing...'
    cd deli && wasm-pack test --headless --chrome

# Runs browser tests for `deli` using firefox (intended for use in CI)
test-firefox-headless:
    @echo 'Testing...'
    cd deli && wasm-pack test --headless --firefox

# Generate readme from doc comments
readme:
    @echo 'Generating README...'
    cd deli && cargo readme > README.md