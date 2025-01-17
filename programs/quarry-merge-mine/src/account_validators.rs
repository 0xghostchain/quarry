//! Account validators

use anchor_lang::prelude::*;
use vipers::validate::Validate;
use vipers::{assert_keys_eq, assert_keys_neq, invariant};

use crate::ClaimRewards;
use crate::WithdrawTokens;
use crate::{InitMergeMiner, QuarryStakePrimary};
use crate::{InitMiner, QuarryStake};
use crate::{NewPool, QuarryStakeReplica};
use anchor_lang::Key;

// --------------------------------
// Instruction account validators
// --------------------------------

impl<'info> Validate<'info> for NewPool<'info> {
    fn validate(&self) -> ProgramResult {
        // replica_mint checks are now redundant
        // since it is now an associated mint
        assert_keys_eq!(
            self.replica_mint.mint_authority.unwrap(),
            self.pool,
            "replica_mint.mint_authority"
        );
        invariant!(
            self.replica_mint.freeze_authority.is_none(),
            "cannot have freeze authority"
        );
        invariant!(
            self.replica_mint.decimals == self.primary_mint.decimals,
            "decimals mismatch"
        );
        invariant!(
            self.replica_mint.supply == 0,
            "replica must have zero supply"
        );

        Ok(())
    }
}

impl<'info> Validate<'info> for InitMergeMiner<'info> {
    fn validate(&self) -> ProgramResult {
        Ok(())
    }
}

impl<'info> Validate<'info> for InitMiner<'info> {
    fn validate(&self) -> ProgramResult {
        require!(
            self.quarry.token_mint_key == self.pool.primary_mint
                || self.quarry.token_mint_key == self.pool.replica_mint,
            InvalidMiner
        );

        assert_keys_eq!(self.mm.pool, self.pool);
        assert_keys_eq!(self.quarry.rewarder_key, self.rewarder);

        assert_keys_eq!(self.miner_vault.owner, self.miner);
        assert_keys_eq!(self.miner_vault.mint, self.quarry.token_mint_key);

        Ok(())
    }
}

impl<'info> Validate<'info> for WithdrawTokens<'info> {
    fn validate(&self) -> ProgramResult {
        let withdraw_mint = self.mm_token_account.mint;

        assert_keys_eq!(self.withdraw_mint, withdraw_mint, "withdraw_mint");
        // cannot withdraw a replica mint.
        assert_keys_neq!(
            self.withdraw_mint.key(),
            self.pool.replica_mint,
            CannotWithdrawReplicaMint
        );

        if withdraw_mint == self.pool.primary_mint {
            // should be no replica balance if withdrawing primary
            require!(self.mm.replica_balance == 0, OutstandingReplicaTokens);
        }

        assert_keys_eq!(self.owner, self.mm.owner);
        assert_keys_eq!(self.pool, self.mm.pool);

        assert_keys_eq!(self.mm_token_account.mint, withdraw_mint);
        assert_keys_eq!(self.mm_token_account.owner, self.mm);

        assert_keys_eq!(
            self.token_destination.mint,
            withdraw_mint,
            "token_destination.mint"
        );

        Ok(())
    }
}

impl<'info> Validate<'info> for ClaimRewards<'info> {
    fn validate(&self) -> ProgramResult {
        self.stake.validate()?;

        assert_keys_eq!(self.minter.mint_wrapper, self.mint_wrapper);
        assert_keys_eq!(self.minter.minter_authority, self.stake.rewarder);
        assert_keys_eq!(self.rewards_token_mint, self.mint_wrapper.token_mint);

        assert_keys_eq!(self.rewards_token_account.mint, self.rewards_token_mint);
        assert_keys_eq!(self.rewards_token_account.owner, self.stake.mm);

        assert_keys_eq!(self.claim_fee_token_account.mint, self.rewards_token_mint);
        assert_keys_eq!(
            self.stake_token_account.mint,
            self.stake.quarry.token_mint_key
        );

        Ok(())
    }
}

/// --------------------------------
/// Context Structs
/// --------------------------------

impl<'info> Validate<'info> for QuarryStakePrimary<'info> {
    fn validate(&self) -> ProgramResult {
        self.stake.validate()?;

        // For primary staking:
        // - quarry is a quarry, staking the `primary_mint`

        assert_keys_eq!(self.mm_owner, self.stake.mm.owner, "mm_owner");
        assert_keys_eq!(
            self.stake.quarry.token_mint_key,
            self.stake.pool.primary_mint,
            "stake.quarry.token_mint_key"
        );

        assert_keys_eq!(
            self.mm_primary_token_account.mint,
            self.stake.pool.primary_mint
        );
        assert_keys_eq!(self.mm_primary_token_account.owner, self.stake.mm);

        Ok(())
    }
}

impl<'info> Validate<'info> for QuarryStakeReplica<'info> {
    fn validate(&self) -> ProgramResult {
        self.stake.validate()?;

        // For replica staking:
        // - rewarder is any rewarder
        // - quarry is a quarry staking the `replica_mint`
        assert_keys_eq!(self.mm_owner, self.stake.mm.owner, "mm_owner");

        assert_keys_eq!(
            self.stake.pool.replica_mint,
            self.replica_mint,
            "stake.pool.replica_mint"
        );
        assert_keys_eq!(
            self.stake.quarry.token_mint_key,
            self.replica_mint,
            "stake.quarry.token_mint_key"
        );

        assert_keys_eq!(self.replica_mint_token_account.mint, self.replica_mint);
        assert_keys_eq!(self.replica_mint_token_account.owner, self.stake.mm);

        Ok(())
    }
}

impl<'info> Validate<'info> for QuarryStake<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys_eq!(self.mm.pool, self.pool, "mm.pool");

        assert_keys_eq!(self.rewarder, self.quarry.rewarder_key);
        assert_keys_eq!(self.quarry, self.miner.quarry_key);
        assert_keys_eq!(self.miner.authority, self.mm, "miner.authority");
        assert_keys_eq!(self.miner_vault, self.miner.token_vault_key);
        assert_keys_eq!(self.miner_vault.owner, self.miner);

        Ok(())
    }
}
