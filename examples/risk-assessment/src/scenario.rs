#[derive(Clone)]
pub(crate) struct Scenario {
    _name: &'static str,
    _category: &'static str,
    /// Percent of TVL (positive = loss, negative = profit)
    pub(crate) loss_pct: f32,
}

pub(crate) fn scenarios() -> Vec<Scenario> {
    let scenarios = vec![
        // Ultra-crisis / systemic
        Scenario {
            _name: "Cross-chain bridge exploit drains collateral",
            _category: "Bridge/Exploit",
            loss_pct: 24.0,
        },
        Scenario {
            _name: "Stablecoin cascade: USDC & USDT -15% for 48h; DAI undercollateralized",
            _category: "Stablecoin/Depeg",
            loss_pct: 22.0,
        },
        Scenario {
            _name: "Sequencer & oracle outage during 40% ETH drop; liquidations halted 3h",
            _category: "Infra/Outage",
            loss_pct: 21.0,
        },
        Scenario {
            _name: "Oracle manipulation + thin liquidity → forced liquidations at stale prices",
            _category: "Oracle/Manipulation",
            loss_pct: 19.5,
        },
        Scenario {
            _name: "Black swan: cascading liquidations, bad debt",
            _category: "Market/Systemic",
            loss_pct: 18.5,
        },
        Scenario {
            _name: "Liquidator insolvency + Dutch auctions underfill",
            _category: "Liquidation/Auction",
            loss_pct: 17.4,
        },
        Scenario {
            _name: "LST slashing + stETH depeg −8%",
            _category: "LST/Depeg",
            loss_pct: 16.2,
        },
        Scenario {
            _name: "DEX liquidity shock post-farm end; slippage on collateral sales",
            _category: "DEX/Liquidity",
            loss_pct: 15.6,
        },
        Scenario {
            _name: "MEV congestion + gas 1500 gwei → failed liquidations",
            _category: "MEV/Gas",
            loss_pct: 14.8,
        },
        Scenario {
            _name: "Severe crash: multiple large position liquidations",
            _category: "Market/Systemic",
            loss_pct: 14.2,
        },
        Scenario {
            _name: "Concentrated whale position triggers market impact",
            _category: "Concentration",
            loss_pct: 13.7,
        },
        Scenario {
            _name: "Cross-chain message delay 6h; remote collateral unprotected",
            _category: "Bridge/Latency",
            loss_pct: 12.9,
        },
        Scenario {
            _name: "Perp funding spike + oracle lag → overstated collateral value",
            _category: "Derivatives/Oracle",
            loss_pct: 12.2,
        },
        Scenario {
            _name: "Major volatility: liquidation delays cause losses",
            _category: "Market/Volatility",
            loss_pct: 11.8,
        },
        Scenario {
            _name: "Stablecoin haircut on treasury assets",
            _category: "Stablecoin/Treasury",
            loss_pct: 11.1,
        },
        Scenario {
            _name: "Keeper network outage 45m",
            _category: "Ops/Keeper",
            loss_pct: 10.6,
        },
        Scenario {
            _name: "Trusted oracle publisher failure; fallback median misprices",
            _category: "Oracle/Infra",
            loss_pct: 9.9,
        },
        Scenario {
            _name: "High volatility: some positions underwater",
            _category: "Market/Volatility",
            loss_pct: 9.3,
        },
        Scenario {
            _name: "L2 congestion: time-to-liquidate exceeds risk half-life",
            _category: "Infra/Congestion",
            loss_pct: 8.8,
        },
        Scenario {
            _name: "Liquidity mining unwind → sell pressure on collateral",
            _category: "DEX/Liquidity",
            loss_pct: 8.1,
        },
        Scenario {
            _name: "NFT-collateral floor -30%",
            _category: "Alt Collateral",
            loss_pct: 7.6,
        },
        Scenario {
            _name: "Moderate crash: efficient liquidations, thin margins",
            _category: "Market",
            loss_pct: 7.1,
        },
        Scenario {
            _name: "Stables yield collapse; reserve revenue shortfall",
            _category: "Revenue",
            loss_pct: 6.4,
        },
        Scenario {
            _name: "Oracle heartbeat lag 60s during fast crash",
            _category: "Oracle/Lag",
            loss_pct: 5.9,
        },
        Scenario {
            _name: "MEV backrun on auctions → worse execution",
            _category: "MEV/Auction",
            loss_pct: 5.3,
        },
        Scenario {
            _name: "LRT (lsETH) depeg −3%",
            _category: "LST/Depeg",
            loss_pct: 4.9,
        },
        Scenario {
            _name: "Minor volatility: small losses from MEV",
            _category: "MEV",
            loss_pct: 4.6,
        },
        Scenario {
            _name: "Partial liquidation loops → extra gas & slippage",
            _category: "Liquidation/Mechanics",
            loss_pct: 4.2,
        },
        Scenario {
            _name: "Cross-venue price divergence; AMM v3 out-of-range",
            _category: "DEX/Mispricing",
            loss_pct: 3.8,
        },
        Scenario {
            _name: "Chain reorg near liquidation block",
            _category: "Infra/Reorg",
            loss_pct: 3.1,
        },
        Scenario {
            _name: "Peg-in/out delay cost on cross-chain collateral",
            _category: "Bridge/Latency",
            loss_pct: 2.7,
        },
        Scenario {
            _name: "Normal volatility: break-even after gas costs",
            _category: "Normal",
            loss_pct: 2.2,
        },
        Scenario {
            _name: "Operational incident: price cap circuit breaker tripped",
            _category: "Ops/Governance",
            loss_pct: 1.7,
        },
        Scenario {
            _name: "Auction dust & leftover bad debt",
            _category: "Liquidation/Auction",
            loss_pct: 1.2,
        },
        Scenario {
            _name: "Stable market: minor operational costs",
            _category: "Normal",
            loss_pct: 0.8,
        },
        Scenario {
            _name: "Oracle fee & infra costs",
            _category: "Ops",
            loss_pct: 0.4,
        },
        Scenario {
            _name: "Baseline: no liquidations needed",
            _category: "Baseline",
            loss_pct: 0.0,
        },
        // Profit / favorable tails (negative = profit)
        Scenario {
            _name: "Liquidation fees exceed losses (high incentive)",
            _category: "Fees",
            loss_pct: -0.6,
        },
        Scenario {
            _name: "Profitable: earned liquidation penalties",
            _category: "Fees",
            loss_pct: -1.2,
        },
        Scenario {
            _name: "MEV rebates on backruns shared",
            _category: "MEV",
            loss_pct: -1.9,
        },
        Scenario {
            _name: "Arbitrage revenue from collateral auctions",
            _category: "Arb",
            loss_pct: -2.4,
        },
        Scenario {
            _name: "Very profitable: high liquidation fees collected",
            _category: "Fees",
            loss_pct: -2.8,
        },
        Scenario {
            _name: "Funding positive carry on borrow side",
            _category: "Revenue",
            loss_pct: -3.5,
        },
        Scenario {
            _name: "Best case: maximum fee collection, no bad debt",
            _category: "Best",
            loss_pct: -4.5,
        },
        Scenario {
            _name: "Insurance partner subsidy payout",
            _category: "Insurance",
            loss_pct: -5.2,
        },
    ];

    let mut scenarios_sorted = scenarios.clone();
    scenarios_sorted.sort_by(|a, b| b.loss_pct.partial_cmp(&a.loss_pct).unwrap());

    scenarios_sorted
}
