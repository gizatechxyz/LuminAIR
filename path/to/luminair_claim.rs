pub struct LuminairClaim {
    pub add: Option<Claim<AddColumn>>,
    pub mul: Option<Claim<MulColumn>>,
    pub lessthan: Option<Claim<LessThanColumn>>,
    pub is_first_log_sizes: Vec<u32>,
}

pub struct LuminairInteractionClaim {
    pub add: Option<InteractionClaim>,
    pub mul: Option<InteractionClaim>,
    pub lessthan: Option<InteractionClaim>,
} 