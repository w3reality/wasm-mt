all: readme

readme: header.md src/lib.rs
	cat header.md > README.md
	cat src/lib.rs | grep '^//!' | sed -E 's/\/\/! ?//g' >> README.md

