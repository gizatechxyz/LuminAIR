import argparse
import json
import torch
from model import PINN
from black_scholes import BlackScholesPINN
from export_weights import export_weights_to_numpy, test_exported_weights


def main(config_path):
    # 1. Load config from JSON file
    with open(config_path, "r") as f:
        config = json.load(f)

    # 2. Train the model
    bs = BlackScholesPINN(config)
    bs.train()

    # 3. Export PyTorch model
    model_path = config.get("model_path", "model.pth")
    bs.export()
    print(f"✅ Model saved to {model_path}")

    # 4. Export weights for Luminal
    print("\n🔄 Exporting weights for Luminal...")
    export_weights_to_numpy(model_path, "weights/")

    # 5. Test the export
    print("\n🧪 Testing weight export...")
    success = test_exported_weights(model_path, "weights/")
    if success:
        print("✅ Weight export test PASSED")
    else:
        print("❌ Weight export test FAILED")

    # 6. Test single prediction with original model
    print("\n🔮 Testing single prediction with original PyTorch model...")
    test_S = torch.tensor([[15.0]], dtype=torch.float32)  # Stock price = 15
    test_t = torch.tensor([[0.5]], dtype=torch.float32)  # Time = 0.5 years

    with torch.no_grad():
        prediction = bs.predict(test_S, test_t)
        print(f"PyTorch prediction for S=15, t=0.5: {prediction.item():.6f}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Train a PINN on the Black-Scholes equation"
    )
    parser.add_argument(
        "--config",
        type=str,
        default="config.json",
        help="Path to the configuration file (default: config.json)",
    )
    args = parser.parse_args()
    main(args.config)
