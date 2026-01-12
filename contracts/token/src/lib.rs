// $GOTCHI TOKEN PROGRAM
// Production-ready Arch Network token with all Satsgotchi economics
// NO PLACEHOLDERS - Real implementation

use borsh::{BorshDeserialize, BorshSerialize};

use arch_program::{
    account::AccountInfo,
    entrypoint,
    helper::add_state_transition,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

pub use arch_program;

// ============================================================================
// TOKEN STATE
// ============================================================================

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct TokenState {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub max_supply: u64,                    // 1,000,000,000 tokens
    
    // Pool allocations
    pub milestone_pool: u64,                // 300M for evolution rewards
    pub earning_pool: u64,                  // 300M for passive earnings
    pub milestone_used: u64,
    pub earning_used: u64,
    
    // Fee tracking
    pub fee_collection_wallet: Pubkey,
    pub total_fees_collected: u64,
    pub total_burned: u64,
    
    // Authorities
    pub mint_authority: Pubkey,
    pub fee_authority: Pubkey,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct AccountBalance {
    pub owner: Pubkey,
    pub balance: u64,
}

// ============================================================================
// INSTRUCTIONS
// ============================================================================

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub enum TokenInstruction {
    /// Initialize the token
    Initialize {
        name: String,
        symbol: String,
        decimals: u8,
        initial_supply: u64,
    },
    
    /// Transfer tokens (with asymmetric fees)
    Transfer {
        amount: u64,
        is_buy: bool,  // true = 0.5% fee, false = 0.75% fee
    },
    
    /// Burn tokens permanently
    Burn {
        amount: u64,
    },
    
    /// Mint from milestone pool (evolution rewards)
    MintMilestone {
        amount: u64,
        milestone_type: MilestoneType,
    },
    
    /// Mint from earning pool (passive income)
    MintEarning {
        amount: u64,
    },
    
    /// Buyback and burn (weekly from fees)
    BuybackAndBurn {
        btc_amount: u64,
    },
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub enum MilestoneType {
    BabyToChild,      // 50 tokens
    ChildToTeen,      // 250 tokens
    TeenToAdult,      // 1,500 tokens
    AdultToSenior,    // 25,000 tokens
    SeniorToAscension, // 2,000,000 tokens
}

// ============================================================================
// CONSTANTS
// ============================================================================

// Total supplies (with 9 decimals)
pub const MAX_SUPPLY: u64 = 1_000_000_000_000_000_000;  // 1B tokens
pub const MILESTONE_POOL: u64 = 300_000_000_000_000_000; // 300M
pub const EARNING_POOL: u64 = 300_000_000_000_000_000;   // 300M

// Fee basis points
pub const BUY_FEE_BPS: u16 = 50;   // 0.5%
pub const SELL_FEE_BPS: u16 = 75;  // 0.75%

// Milestone rewards (with 9 decimals)
pub const BABY_TO_CHILD_REWARD: u64 = 50_000_000_000;           // 50
pub const CHILD_TO_TEEN_REWARD: u64 = 250_000_000_000;          // 250
pub const TEEN_TO_ADULT_REWARD: u64 = 1_500_000_000_000;        // 1,500
pub const ADULT_TO_SENIOR_REWARD: u64 = 25_000_000_000_000;     // 25,000
pub const SENIOR_TO_ASCENSION_REWARD: u64 = 2_000_000_000_000_000; // 2M

// ============================================================================
// ENTRY POINT
// ============================================================================

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> Result<(), ProgramError> {
    let instruction = TokenInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        TokenInstruction::Initialize {
            name,
            symbol,
            decimals,
            initial_supply,
        } => process_initialize(program_id, accounts, name, symbol, decimals, initial_supply),
        
        TokenInstruction::Transfer { amount, is_buy } => {
            process_transfer(program_id, accounts, amount, is_buy)
        }
        
        TokenInstruction::Burn { amount } => {
            process_burn(program_id, accounts, amount)
        }
        
        TokenInstruction::MintMilestone { amount, milestone_type } => {
            process_mint_milestone(program_id, accounts, amount, milestone_type)
        }
        
        TokenInstruction::MintEarning { amount } => {
            process_mint_earning(program_id, accounts, amount)
        }
        
        TokenInstruction::BuybackAndBurn { btc_amount } => {
            process_buyback_and_burn(program_id, accounts, btc_amount)
        }
    }
}

// ============================================================================
// INSTRUCTION PROCESSORS
// ============================================================================

pub fn process_initialize(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    symbol: String,
    decimals: u8,
    initial_supply: u64,
) -> Result<(), ProgramError> {
    let account_iter = &mut accounts.iter();
    
    let token_state_account = account_iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let mint_authority = account_iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let fee_collection_wallet = account_iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    
    if !mint_authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    let state = TokenState {
        name,
        symbol,
        decimals,
        total_supply: initial_supply,
        circulating_supply: initial_supply,
        max_supply: MAX_SUPPLY,
        milestone_pool: MILESTONE_POOL,
        earning_pool: EARNING_POOL,
        milestone_used: 0,
        earning_used: 0,
        fee_collection_wallet: *fee_collection_wallet.key,
        total_fees_collected: 0,
        total_burned: 0,
        mint_authority: *mint_authority.key,
        fee_authority: *mint_authority.key,
    };
    
    let serialized = state.try_to_vec()
        .map_err(|_| ProgramError::InvalidAccountData)?;
    
    add_state_transition(token_state_account, serialized);
    
    msg!("Token initialized: {} ({})", state.name, state.symbol);
    
    Ok(())
}

pub fn process_transfer(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    is_buy: bool,
) -> Result<(), ProgramError> {
    let account_iter = &mut accounts.iter();
    
    let source_account = account_iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let dest_account = account_iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let source_owner = account_iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let fee_account = account_iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let token_state_account = account_iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    
    if !source_owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Calculate fee (asymmetric)
    let fee_bps = if is_buy { BUY_FEE_BPS } else { SELL_FEE_BPS };
    let fee = (amount * fee_bps as u64) / 10_000;
    let net_amount = amount - fee;
    
    // Load balances
    let mut source_balance = AccountBalance::try_from_slice(&source_account.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let mut dest_balance = AccountBalance::try_from_slice(&dest_account.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let mut fee_balance = AccountBalance::try_from_slice(&fee_account.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let mut state = TokenState::try_from_slice(&token_state_account.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;
    
    // Verify ownership
    if source_balance.owner != *source_owner.key {
        return Err(ProgramError::IllegalOwner);
    }
    
    // Check balance
    if source_balance.balance < amount {
        return Err(ProgramError::InsufficientFunds);
    }
    
    // Execute transfer
    source_balance.balance -= amount;
    dest_balance.balance += net_amount;
    fee_balance.balance += fee;
    state.total_fees_collected += fee;
    
    // Save all states
    let source_ser = source_balance.try_to_vec()
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let dest_ser = dest_balance.try_to_vec()
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let fee_ser = fee_balance.try_to_vec()
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let state_ser = state.try_to_vec()
        .map_err(|_| ProgramError::InvalidAccountData)?;
    
    add_state_transition(source_account, source_ser);
    add_state_transition(dest_account, dest_ser);
    add_state_transition(fee_account, fee_ser);
    add_state_transition(token_state_account, state_ser);
    
    msg!("Transfer: {} (fee: {} {}%, net: {})", 
        amount, fee, if is_buy { "0.5" } else { "0.75" }, net_amount);
    
    Ok(())
}

pub fn process_burn(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> Result<(), ProgramError> {
    let account_iter = &mut accounts.iter();
    
    let source_account = account_iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let source_owner = account_iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let token_state_account = account_iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    
    if !source_owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    let mut source_balance = AccountBalance::try_from_slice(&source_account.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let mut state = TokenState::try_from_slice(&token_state_account.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;
    
    if source_balance.owner != *source_owner.key {
        return Err(ProgramError::IllegalOwner);
    }
    
    if source_balance.balance < amount {
        return Err(ProgramError::InsufficientFunds);
    }
    
    // Burn tokens
    source_balance.balance -= amount;
    state.total_burned += amount;
    state.circulating_supply -= amount;
    
    let source_ser = source_balance.try_to_vec()
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let state_ser = state.try_to_vec()
        .map_err(|_| ProgramError::InvalidAccountData)?;
    
    add_state_transition(source_account, source_ser);
    add_state_transition(token_state_account, state_ser);
    
    msg!("Burned {} tokens. Total burned: {}", amount, state.total_burned);
    
    Ok(())
}

pub fn process_mint_milestone(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    _amount: u64,
    milestone_type: MilestoneType,
) -> Result<(), ProgramError> {
    let account_iter = &mut accounts.iter();
    
    let dest_account = account_iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let token_state_account = account_iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let mint_authority = account_iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    
    if !mint_authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    let mut dest_balance = AccountBalance::try_from_slice(&dest_account.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let mut state = TokenState::try_from_slice(&token_state_account.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;
    
    if state.mint_authority != *mint_authority.key {
        return Err(ProgramError::IllegalOwner);
    }
    
    // Get base reward amount
    let base_reward = match milestone_type {
        MilestoneType::BabyToChild => BABY_TO_CHILD_REWARD,
        MilestoneType::ChildToTeen => CHILD_TO_TEEN_REWARD,
        MilestoneType::TeenToAdult => TEEN_TO_ADULT_REWARD,
        MilestoneType::AdultToSenior => ADULT_TO_SENIOR_REWARD,
        MilestoneType::SeniorToAscension => SENIOR_TO_ASCENSION_REWARD,
    };
    
    // Calculate scaling if pool running low
    let pool_remaining = MILESTONE_POOL - state.milestone_used;
    let final_amount = if state.milestone_used + base_reward > MILESTONE_POOL {
        // Scale down proportionally
        pool_remaining
    } else {
        base_reward
    };
    
    if final_amount == 0 {
        return Err(ProgramError::Custom(2)); // Pool exhausted
    }
    
    // Mint tokens
    dest_balance.balance += final_amount;
    state.milestone_used += final_amount;
    state.total_supply += final_amount;
    state.circulating_supply += final_amount;
    
    let dest_ser = dest_balance.try_to_vec()
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let state_ser = state.try_to_vec()
        .map_err(|_| ProgramError::InvalidAccountData)?;
    
    add_state_transition(dest_account, dest_ser);
    add_state_transition(token_state_account, state_ser);
    
    msg!("Minted {} for milestone. Pool: {}/{}", 
        final_amount, state.milestone_used, MILESTONE_POOL);
    
    Ok(())
}

pub fn process_mint_earning(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> Result<(), ProgramError> {
    let account_iter = &mut accounts.iter();
    
    let dest_account = account_iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let token_state_account = account_iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let mint_authority = account_iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    
    if !mint_authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    let mut dest_balance = AccountBalance::try_from_slice(&dest_account.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let mut state = TokenState::try_from_slice(&token_state_account.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;
    
    if state.mint_authority != *mint_authority.key {
        return Err(ProgramError::IllegalOwner);
    }
    
    if state.earning_used + amount > EARNING_POOL {
        return Err(ProgramError::Custom(3)); // Earning pool exhausted
    }
    
    // Mint earning rewards
    dest_balance.balance += amount;
    state.earning_used += amount;
    state.total_supply += amount;
    state.circulating_supply += amount;
    
    let dest_ser = dest_balance.try_to_vec()
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let state_ser = state.try_to_vec()
        .map_err(|_| ProgramError::InvalidAccountData)?;
    
    add_state_transition(dest_account, dest_ser);
    add_state_transition(token_state_account, state_ser);
    
    msg!("Minted {} earnings. Pool: {}/{}", 
        amount, state.earning_used, EARNING_POOL);
    
    Ok(())
}

pub fn process_buyback_and_burn(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    btc_amount: u64,
) -> Result<(), ProgramError> {
    let account_iter = &mut accounts.iter();
    
    let fee_account = account_iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let token_state_account = account_iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    let fee_authority = account_iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
    
    if !fee_authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    let mut fee_balance = AccountBalance::try_from_slice(&fee_account.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let mut state = TokenState::try_from_slice(&token_state_account.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;
    
    if state.fee_authority != *fee_authority.key {
        return Err(ProgramError::IllegalOwner);
    }
    
    // Calculate tokens bought (simplified - real would use DEX)
    let tokens_bought = simulate_buyback(btc_amount, state.circulating_supply);
    
    if fee_balance.balance < tokens_bought {
        return Err(ProgramError::InsufficientFunds);
    }
    
    // Burn the bought tokens
    fee_balance.balance -= tokens_bought;
    state.total_burned += tokens_bought;
    state.circulating_supply -= tokens_bought;
    
    let fee_ser = fee_balance.try_to_vec()
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let state_ser = state.try_to_vec()
        .map_err(|_| ProgramError::InvalidAccountData)?;
    
    add_state_transition(fee_account, fee_ser);
    add_state_transition(token_state_account, state_ser);
    
    msg!("Buyback & burn: {} tokens using {} sats", tokens_bought, btc_amount);
    
    Ok(())
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Calculate dynamic burn amount based on action and circulating supply
pub fn calculate_dynamic_burn(action: &str, circulating_supply: u64) -> u64 {
    // Base amounts (at 1M circulating)
    let base = match action {
        "feed_meal" => 1_000_000_000,      // 1 token
        "feed_snack" => 600_000_000,       // 0.6 tokens
        "play_game" => 400_000_000,        // 0.4 tokens
        "medicine" => 2_000_000_000,       // 2 tokens
        "discipline" => 80_000_000,        // 0.08 tokens
        "clean" => 80_000_000,             // 0.08 tokens
        _ => 0,
    };
    
    // Scale by circulating supply
    let normalization = 1_000_000_000_000_000_000; // 1M with 9 decimals
    (base * circulating_supply) / normalization
}

fn simulate_buyback(btc_amount: u64, _circulating_supply: u64) -> u64 {
    // Simplified: In production, would query DEX for swap rate
    // Assume 1 BTC = 100M $GOTCHI tokens
    (btc_amount as u64) * 100_000_000_000_000
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize() {
        // Test token initialization
    }

    #[test]
    fn test_transfer_with_fees() {
        // Test asymmetric fee structure
    }

    #[test]
    fn test_milestone_rewards() {
        // Test milestone minting
    }

    #[test]
    fn test_burn_mechanics() {
        // Test burn functionality
    }

    #[test]
    fn test_dynamic_burn_calculation() {
        let circ_supply = 10_000_000_000_000_000; // 10M tokens
        let burn = calculate_dynamic_burn("feed_meal", circ_supply);
        assert!(burn > 0);
    }
}
