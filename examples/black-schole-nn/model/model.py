import torch
import torch.nn as nn
from utils import * 


class PINN(nn.Module):
    def __init__(self):
        super(PINN, self).__init__()
        self.net = nn.Sequential(
            nn.Linear(2, 64),
            nn.Tanh(),
            nn.Linear(64, 64),
            nn.Tanh(),
            nn.Linear(64, 1)
        )
        

    def forward(self, S, t):
        return self.net(torch.cat([S, t], dim=1))
    

    
