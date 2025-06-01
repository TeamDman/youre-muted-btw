# https://github.com/johnthagen/min-sized-rust
$old=$env:RUSTFLAGS
$Env:RUSTFLAGS="-Zlocation-detail=none -Zfmt-debug=none"
try {
    cargo build --release
} finally {
    $env:RUSTFLAGS=$old
}