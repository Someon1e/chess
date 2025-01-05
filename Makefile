OS := $(shell uname)

# Detect target triple
TARGET := $(shell rustup show | grep "Default host" | cut -d' ' -f3)

# Paths and settings
PGO_DIR := engine_pgo
ENGINE_DIR := engine
OUTPUT_DIR := $(ENGINE_DIR)/target/$(TARGET)/release

# Default executable name (can be overridden with EXE)
EXE ?= built
EXECUTABLE := $(OUTPUT_DIR)/engine
FINAL_EXE := $(EXE)

# Default option
all: build

# Build the project with PGO
build:
	cd $(PGO_DIR) && cargo run --release -- $(TARGET)
	mv $(EXECUTABLE) ./$(FINAL_EXE)

# Run the main executable
run: build
	./$(FINAL_EXE)

# Clean up the build artifacts
clean:
	cargo clean --manifest-path $(PGO_DIR)/Cargo.toml
	cargo clean --manifest-path $(ENGINE_DIR)/Cargo.toml
	if [ -f ./$(FINAL_EXE) ]; then rm ./$(FINAL_EXE); fi
