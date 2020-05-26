all: header.md src/lib.rs
	echo '<!-- ⚠️  THIS IS A GENERATED FILE -->' > README.md
	cat header.md >> README.md
	cat src/lib.rs | grep '^//!' | sed -E 's/\/\/! ?//g' >> README.md

