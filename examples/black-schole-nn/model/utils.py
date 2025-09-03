import numpy as np
from scipy.stats import norm
import torch

def black_scholes_solution(S, K, T, r, sigma):
    S = np.array(S)
    d1 = (np.log(S / K) + (r + 0.5 * sigma**2) * T) / (sigma * np.sqrt(T))
    d2 = d1 - sigma * np.sqrt(T)
    return S * norm.cdf(d1) - K * np.exp(-r * T) * norm.cdf(d2)


def prepare_dataset(config):
    S_eval = torch.linspace(config["min_S"], config["max_S"], 200).view(-1, 1)
    t_eval = torch.zeros_like(S_eval)
    return S_eval, t_eval