test:
	RUST_BACKTRACE=1 cargo test --release --color=always --package auction-sp1-prover --lib tests::test_sp1_prover --no-fail-fast -- --exact -Z unstable-options --show-output
	
gen-key:
	cargo test --color=always --package auction-sp1-prover --lib tests::test_gen_key --no-fail-fast -- --exact -Z unstable-options --show-output

elf-vk:
	cargo prove build
	cargo prove vkey --elf elf/riscv32im-succinct-zkvm-elf > elf/elf_verifying_key
