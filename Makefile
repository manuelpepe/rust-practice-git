build: 
	rustc src/main.rs --out-dir build/

.PHONY: build clean 

clean:
	rm -r build/*
