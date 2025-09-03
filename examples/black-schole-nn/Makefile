# Black-Scholes Neural Network Makefile
# This Makefile provides targets for setup, training, and prediction with Luminair

.PHONY: help setup train predict clean build

# Default target
help:
	@echo "Available targets:"
	@echo "  setup   - Install Python dependencies and create environment"
	@echo "  train   - Train the PyTorch model and export weights"
	@echo "  predict - Run predictions with the Rust/Luminair model"
	@echo "  build   - Build the Rust executable"
	@echo "  clean   - Clean generated files and cache"

# Setup target: Install dependencies
setup:
	@echo "🔧 Setting up environment..."
	pip install -e .
	@echo "📦 Installing development dependencies..."
	pip install -e ".[dev]"
	@echo "📁 Creating weights directory..."
	mkdir -p weights
	@echo "✅ Setup complete!"

# Training target: Train the model and export weights
train:
	@echo "🚀 Starting training..."
	cd model && python main.py --config config.json
	@echo "✅ Training complete!"

# Build Rust executable
build:
	@echo "🔨 Building Rust executable..."
	cargo build --release
	@echo "✅ Build complete!"

# Run predictions with Rust/Luminair
predict: build
	@echo "🔮 Running predictions with Rust/Luminair model..."
	cargo run --release
	@echo "✅ Prediction complete!"

# Clean target: Remove generated files
clean:
	@echo "🧹 Cleaning up..."
	rm -rf target/
	rm -rf weights/
	rm -f model.pth
	rm -rf __pycache__/
	rm -rf model/__pycache__/
	rm -rf *.egg-info/
	rm -rf build/
	rm -rf dist/
	find . -name "*.pyc" -delete
	find . -name ".DS_Store" -delete
	@echo "✅ Cleanup complete!"

# Full workflow: setup, train, and predict
all: setup train predict
	@echo "✅ Full workflow complete!"