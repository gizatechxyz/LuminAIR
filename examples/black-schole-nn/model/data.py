import numpy as np
import torch
from utils import black_scholes_solution

def generate_synthetic_data(config):
    S = np.random.uniform(config["min_S"], config["max_S"], (config["N_data"], 1))
    t = np.random.uniform(0, config["T"], (config["N_data"], 1))
    C = black_scholes_solution(S, config["K"], config["T"] - t, config["r"], config["sigma"])
    C += np.random.normal(config["bias"], config["noise_variance"], size=C.shape)
    return (
        torch.tensor(S, dtype=torch.float32, requires_grad=True),
        torch.tensor(t, dtype=torch.float32, requires_grad=True),
        torch.tensor(C, dtype=torch.float32),
    )

def generate_collocation_points(config, N_colloc=1000):
    S = torch.tensor(np.random.uniform(config["min_S"], config["max_S"], (N_colloc, 1)), dtype=torch.float32, requires_grad=True)
    t = torch.tensor(np.random.uniform(0, config["T"], (N_colloc, 1)), dtype=torch.float32, requires_grad=True)
    return S, t
