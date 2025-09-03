from data import generate_collocation_points
from loss import total_loss

def train(model, optimizer, config, S_data, t_data, C_data):
    for epoch in range(config["epochs"]):
        optimizer.zero_grad()
        S_colloc, t_colloc = generate_collocation_points(config)
        loss, loss_data, loss_pde = total_loss(model, S_data, t_data, C_data, S_colloc, t_colloc, config["r"], config["sigma"])
        loss.backward()
        optimizer.step()

        if epoch % 200 == 0:
            print(f"Epoch {epoch} | Total: {loss.item():.6f} | Data: {loss_data.item():.6f} | PDE: {loss_pde.item():.6f}")
