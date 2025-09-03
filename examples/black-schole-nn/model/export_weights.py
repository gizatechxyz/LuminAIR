import torch
import numpy as np
import json
import os
from model import PINN

def _to_numpy_T(t: torch.Tensor) -> np.ndarray:
    # Ensure (in, out) layout and contiguous memory
    return t.detach().cpu().t().contiguous().numpy()

def _to_numpy(t: torch.Tensor) -> np.ndarray:
    return t.detach().cpu().contiguous().numpy()

def export_weights_to_numpy(model_path="model.pth", output_dir="weights/"):
    """Export PyTorch model weights to numpy arrays for Luminal, with weights TRANSPOSED."""
    os.makedirs(output_dir, exist_ok=True)

    model = PINN()
    state = torch.load(model_path, map_location="cpu")
    model.load_state_dict(state)
    model.eval()

    # Collect weights (transposed) and biases
    weights = {
        # PyTorch Linear weight is (out, in); we save as (in, out)
        "layer1_weight": _to_numpy_T(model.net[0].weight),
        "layer1_bias":   _to_numpy(model.net[0].bias),

        "layer2_weight": _to_numpy_T(model.net[2].weight),
        "layer2_bias":   _to_numpy(model.net[2].bias),

        "layer3_weight": _to_numpy_T(model.net[4].weight),
        "layer3_bias":   _to_numpy(model.net[4].bias),
    }

    for name, arr in weights.items():
        np.save(os.path.join(output_dir, f"{name}.npy"), arr)
        print(f"Saved {name}: shape {arr.shape}, dtype {arr.dtype}")

    # Shapes info (both original PyTorch shapes and saved shapes)
    shapes_info = {
        "layer1": {
            "pytorch_weight": [model.net[0].weight.shape[0], model.net[0].weight.shape[1]],  # (out, in)
            "saved_weight":   [weights["layer1_weight"].shape[0], weights["layer1_weight"].shape[1]],  # (in, out)
            "bias":           list(weights["layer1_bias"].shape)
        },
        "layer2": {
            "pytorch_weight": [model.net[2].weight.shape[0], model.net[2].weight.shape[1]],
            "saved_weight":   [weights["layer2_weight"].shape[0], weights["layer2_weight"].shape[1]],
            "bias":           list(weights["layer2_bias"].shape)
        },
        "layer3": {
            "pytorch_weight": [model.net[4].weight.shape[0], model.net[4].weight.shape[1]],
            "saved_weight":   [weights["layer3_weight"].shape[0], weights["layer3_weight"].shape[1]],
            "bias":           list(weights["layer3_bias"].shape)
        }
    }

    with open(os.path.join(output_dir, "model_info.json"), "w") as f:
        json.dump(shapes_info, f, indent=2)

    print(f"All weights exported (transposed) to {output_dir}")
    return weights

def test_exported_weights(model_path="model.pth", weights_dir="weights/"):
    """Verify exported (transposed) weights reproduce the model outputs."""
    model = PINN()
    model.load_state_dict(torch.load(model_path, map_location="cpu"))
    model.eval()

    # Load exported NP arrays (weights already transposed to (in, out))
    w1 = np.load(os.path.join(weights_dir, "layer1_weight.npy"))  # (2, 64)
    b1 = np.load(os.path.join(weights_dir, "layer1_bias.npy"))    # (64,)
    w2 = np.load(os.path.join(weights_dir, "layer2_weight.npy"))  # (64, 64)
    b2 = np.load(os.path.join(weights_dir, "layer2_bias.npy"))    # (64,)
    w3 = np.load(os.path.join(weights_dir, "layer3_weight.npy"))  # (64, 1)
    b3 = np.load(os.path.join(weights_dir, "layer3_bias.npy"))    # (1,)

    # Test input
    test_S = torch.tensor([[15.0]], dtype=torch.float32)
    test_t = torch.tensor([[0.5]], dtype=torch.float32)

    with torch.no_grad():
        original = model(test_S, test_t).item()

    # Manual forward (NO transpose now)
    x = torch.cat([test_S, test_t], dim=1).numpy()  # (1, 2)

    x = x @ w1 + b1           # (1, 64)
    x = np.tanh(x)
    x = x @ w2 + b2           # (1, 64)
    x = np.tanh(x)
    x = x @ w3 + b3           # (1, 1)

    manual = float(x[0, 0])
    diff = abs(original - manual)

    print(f"Original: {original}")
    print(f"Manual:   {manual}")
    print(f"Diff:     {diff}")
    return diff < 1e-6

if __name__ == "__main__":
    export_weights_to_numpy()
    ok = test_exported_weights()
    print(f"Weight export test: {'PASSED' if ok else 'FAILED'}")
