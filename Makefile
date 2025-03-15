# Paths
PGO_DIR := engine_pgo
ENGINE_DIR := encrustant

EXE ?= built
TARGET ?=

# Default option
all: build

# Build the project with PGO
build:
	cd $(PGO_DIR) && cargo run --release -- $(if $(TARGET),-t $(TARGET)) $(if $(EXE),-o ../$(EXE))

# Run the main executable
run: build
	./$(EXE)

# Clean up the build artifacts
clean:
	cargo clean --manifest-path $(PGO_DIR)/Cargo.toml
	cargo clean --manifest-path $(ENGINE_DIR)/Cargo.toml
	if [ -f ./$(EXE) ]; then rm ./$(EXE); fi
