# user_name        := env("USER")
# current_location := justfile()
current_dir      := justfile_directory()
module_name      := file_name(current_dir)
target_dir       := `cargo metadata --no-deps --format-version=1 | jq -r '.target_directory'`

default: build

build:
    cargo build
    RUST_BACKTRACE=1 cargo test
    cargo clippy

fix:
    @RUST_BACKTRACE=1 aifix -l rust -t fix_code -f {{current_dir}} -f {{current_dir}}/..

fixd:
    @RUST_BACKTRACE=1 aifix -d -l rust -t fix_code -f {{current_dir}} -f {{current_dir}}/..

fixws:
    @RUST_BACKTRACE=1 aifix -l rust -t fix_code -w ~/svn/_workspace -f {{current_dir}} -f {{current_dir}}/.. -f ~/svn/_workspace

doc_item:
    RUST_BACKTRACE=1 aifix -l rust -t write_item_doc -f {{current_dir}} -f {{current_dir}}/..

docws_item:
    RUST_BACKTRACE=1 aifix -l rust -t write_item_doc -w ~/svn/_workspace -f {{current_dir}} -f {{current_dir}}/..

doc_block:
    RUST_BACKTRACE=1 aifix -l rust -t write_block_doc -f {{current_dir}} -f {{current_dir}}/..

doc_module:
    RUST_BACKTRACE=1 aifix -l rust -t write_module_doc -f {{current_dir}} -f {{current_dir}}/..

tclean:
    @cargo clean -p {{module_name}}

clean-all:
    @rm target -rf
    @cargo clean

targetlist:
    rustup target list

rpi:
    cargo build --target aarch64-unknown-linux-gnu

cover-setup:
    cargo install cargo-llvm-cov
    rustup component add llvm-tools-preview

cover:
	cargo llvm-cov --all-features --workspace --html

cover-lcov:
	cargo llvm-cov --all-features --workspace --lcov --output-path {{target_dir}}/coverage/lcov.info

cover-text:
	cargo llvm-cov --all-features --workspace

