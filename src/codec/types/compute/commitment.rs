use std::{error::Error, io::Cursor};

use num_bigint::BigUint;
use num_rational::BigRational;

use crate::cursor::{read_perbill, read_u128, read_u256, read_u32, CursorExt};

use super::stake::Stake;

#[derive(uniffi::Record, Debug, PartialEq)]
pub struct Commitment {
    pub stake: Option<Stake>,
    pub commission: BigRational,
    pub delegations_total_amount: BigUint,
    pub delegations_total_rewardable_amount: BigUint,
    pub weights: WeightsMemoryBuffer,
    pub pool_rewards: PoolRewardsMemoryBuffer,
    pub last_scoring_epoch: u32,
    pub last_slashing_epoch: u32,
}

impl Commitment {
    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let stake = cursor.read_optional(|c| Stake::read(c))?;
        let commission = read_perbill(cursor)?;
        let delegations_total_amount = read_u128(cursor)?;
        let delegations_total_rewardable_amount = read_u128(cursor)?;
        let weights = WeightsMemoryBuffer::read(cursor)?;
        let pool_rewards = PoolRewardsMemoryBuffer::read(cursor)?;
        let last_scoring_epoch = read_u32(cursor)?;
        let last_slashing_epoch = read_u32(cursor)?;

        Ok(Self {
            stake,
            commission,
            delegations_total_amount,
            delegations_total_rewardable_amount,
            weights,
            pool_rewards,
            last_scoring_epoch,
            last_slashing_epoch,
        })
    }
}

#[derive(uniffi::Record, Debug, PartialEq)]
pub struct CommitmentWeights {
    pub self_reward_weight: BigUint,
    pub self_slash_weight: BigUint,
    pub delegations_reward_weight: BigUint,
    pub delegations_slash_weight: BigUint,
}

impl CommitmentWeights {
    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let self_reward_weight = read_u256(cursor)?;
        let self_slash_weight = read_u256(cursor)?;
        let delegations_reward_weight = read_u256(cursor)?;
        let delegations_slash_weight = read_u256(cursor)?;

        Ok(Self {
            self_reward_weight,
            self_slash_weight,
            delegations_reward_weight,
            delegations_slash_weight,
        })
    }
}

#[derive(uniffi::Record, Debug, PartialEq)]
pub struct PoolReward {
    pub reward_per_weight: BigUint,
    pub slash_per_weight: BigUint,
}

impl PoolReward {
    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let reward_per_weight = read_u256(cursor)?;
        let slash_per_weight = read_u256(cursor)?;

        Ok(Self {
            reward_per_weight,
            slash_per_weight,
        })
    }
}

pub trait MemoryBufferItem {
    type K;
    type V;

    fn new(key: Self::K, value: Self::V) -> Self;

    fn read_key(cursor: &mut Cursor<Vec<u8>>) -> Result<Self::K, Box<dyn Error>>;
    fn read_value(cursor: &mut Cursor<Vec<u8>>) -> Result<Self::V, Box<dyn Error>>;
    fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> where Self: Sized {
        let key = Self::read_key(cursor)?;
        let value = Self::read_value(cursor)?;

        Ok(Self::new(key, value))
    }
}

pub trait MemoryBuffer {
    type T: MemoryBufferItem;

    fn new(past: Option<Self::T>, current: Self::T) -> Self;

    fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<Self, Box<dyn Error>> where Self: Sized {
        let past = cursor.read_optional(|c| Self::T::read(c))?;
        let current = Self::T::read(cursor)?;

        Ok(Self::new(past, current))
    }
}

#[derive(uniffi::Record, Debug, PartialEq)]
pub struct WeightsMemoryBufferItem {
    pub timestamp: u32,
    pub weights: CommitmentWeights,
}

impl MemoryBufferItem for WeightsMemoryBufferItem {
    type K = u32;
    type V = CommitmentWeights;

    fn read_key(cursor: &mut Cursor<Vec<u8>>) -> Result<Self::K, Box<dyn Error>> {
        read_u32(cursor)
    }

    fn read_value(cursor: &mut Cursor<Vec<u8>>) -> Result<Self::V, Box<dyn Error>> {
        CommitmentWeights::read(cursor)
    }

    fn new(key: u32, value: CommitmentWeights) -> Self {
        Self {
            timestamp: key,
            weights: value,
        }
    }
}

#[derive(uniffi::Record, Debug, PartialEq)]
pub struct WeightsMemoryBuffer {
    pub past: Option<WeightsMemoryBufferItem>,
    pub current: WeightsMemoryBufferItem,
}

impl MemoryBuffer for WeightsMemoryBuffer {
    type T = WeightsMemoryBufferItem;

    fn new(past: Option<Self::T>, current: Self::T) -> Self {
        Self {
            past,
            current,
        }
    }
}

#[derive(uniffi::Record, Debug, PartialEq)]
pub struct PoolRewardsMemoryBufferItem {
    pub timestamp: u32,
    pub pool_reward: PoolReward,
}

impl MemoryBufferItem for PoolRewardsMemoryBufferItem {
    type K = u32;
    type V = PoolReward;

    fn read_key(cursor: &mut Cursor<Vec<u8>>) -> Result<Self::K, Box<dyn Error>> {
        read_u32(cursor)
    }

    fn read_value(cursor: &mut Cursor<Vec<u8>>) -> Result<Self::V, Box<dyn Error>> {
        PoolReward::read(cursor)
    }

    fn new(key: u32, value: PoolReward) -> Self {
        Self {
            timestamp: key,
            pool_reward: value,
        }
    }
}

#[derive(uniffi::Record, Debug, PartialEq)]
pub struct PoolRewardsMemoryBuffer {
    pub past: Option<PoolRewardsMemoryBufferItem>,
    pub current: PoolRewardsMemoryBufferItem,
}

impl MemoryBuffer for PoolRewardsMemoryBuffer {
    type T = PoolRewardsMemoryBufferItem;

    fn new(past: Option<Self::T>, current: Self::T) -> Self {
        Self {
            past,
            current,
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use num_bigint::BigInt;
    use num_traits::FromPrimitive;

    use crate::hex::hex_to_ba;

    use super::*;

    #[test]
    fn test_decode_commitment() {
        let encoded = "0x010080145e6e2f2e0000000000000000000080145e6e2f2e000000000000000000a9a9860080700000007a7fa639b1520000000000000000000000000000000000000000000000000000006e58f8ca880a0000000000000000000000000000000000000000000000000000000000002680c85e0ab789010000000000000000260023f8d450c600000000000000000001c12600000080145e6e2f2e000000000000000000000000000000000000000000000000000080145e6e2f2e0000000000000000000000000000000000000000000000000026705f1b8108c50000000000000000000000000000000000000000000000000026f00482b66e8801000000000000000000000000000000000000000000000000c22600000080145e6e2f2e000000000000000000000000000000000000000000000000000080145e6e2f2e00000000000000000000000000000000000000000000000000260023f8d450c6000000000000000000000000000000000000000000000000002680c85e0ab78901000000000000000000000000000000000000000000000000016c2d7a00d3a9de1531849c4fcd83e68e000000000000000000000000000000000000000094b31d768524024f4116459c0000000000000000000000000000000000000000a9a98600db10eb27ba847d93b170380f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000c426000042260000";
        let encoded = hex_to_ba(encoded);
        let mut cursor = Cursor::new(encoded);
        let commitment = Commitment::read(&mut cursor).unwrap();

        assert_eq!(
            Commitment {
                stake: Some(Stake {
                    amount: BigUint::from_str("13000000000000000").unwrap(),
                    rewardable_amount: BigUint::from_str("13000000000000000").unwrap(),
                    created: 8_825_257,
                    cooldown_period: 28_800,
                    cooldown_started: None,
                    accrued_reward: BigUint::from_str("90921129901946").unwrap(),
                    accrued_slash: BigUint::from_str("0").unwrap(),
                    allow_auto_compound: false,
                    paid: BigUint::from_str("11582637103214").unwrap(),
                    applied_slash: BigUint::from_str("0").unwrap(),
                }),
                commission: BigRational::from_f32(0.0).unwrap(),
                delegations_total_amount: BigUint::from_str("110820921015042086").unwrap(),
                delegations_total_rewardable_amount: BigUint::from_str("55820921015042086").unwrap(),
                weights: WeightsMemoryBuffer {
                    past: Some(WeightsMemoryBufferItem {
                        timestamp: 9_921,
                        weights: CommitmentWeights {
                            self_reward_weight: BigUint::from_str("13000000000000000").unwrap(),
                            self_slash_weight: BigUint::from_str("13000000000000000").unwrap(),
                            delegations_reward_weight: BigUint::from_str("55459921015042086").unwrap(),
                            delegations_slash_weight: BigUint::from_str("110459921015042086").unwrap(),
                        }
                    }),
                    current: WeightsMemoryBufferItem {
                        timestamp: 9_922,
                        weights: CommitmentWeights {
                            self_reward_weight: BigUint::from_str("13000000000000000").unwrap(),
                            self_slash_weight: BigUint::from_str("13000000000000000").unwrap(),
                            delegations_reward_weight: BigUint::from_str("55820921015042086").unwrap(),
                            delegations_slash_weight: BigUint::from_str("110820921015042086").unwrap(),
                        }
                    },
                },
                pool_rewards: PoolRewardsMemoryBuffer {
                    past: Some(PoolRewardsMemoryBufferItem {
                        timestamp: 8_007_020,
                        pool_reward: PoolReward {
                            reward_per_weight: BigUint::from_str("44225546750470760874800753107").unwrap(),
                            slash_per_weight: BigUint::from_str("48363182510477381092016763796").unwrap(),
                        }
                    }),
                    current: PoolRewardsMemoryBufferItem {
                        timestamp: 8_825_257,
                        pool_reward: PoolReward {
                            reward_per_weight: BigUint::from_str("4710507173966174342242570459").unwrap(),
                            slash_per_weight: BigUint::from_str("0").unwrap(),
                        }
                    },
                },
                last_scoring_epoch: 9_924,
                last_slashing_epoch: 9_794,
            },
            commitment,
        );

        let encoded = "0x010b25cabf076a550000000000000000000b25cabf076a55000000000000000000398182008070000000f99b15409f0c00000000000000000000000000000000000000000000000000000178ded3a876a4040000000000000000000000000000000000000000000000000080f0fa02c7e8a6e36f81d20000000000000000007fa0619b29eccf00000000000000000001bd2600000b4505d56a0550000000000000000000000000000000000000000000000000000b4505d56a055000000000000000000000000000000000000000000000000000071133cdd558b0000000000000000000000000000000000000000000000000009034c3fe17ceb100000000000000000000000000000000000000000000000000c22600000b25cabf076a55000000000000000000000000000000000000000000000000000b25cabf076a5500000000000000000000000000000000000000000000000000954dce1ef476b0000000000000000000000000000000000000000000000000001e715e5036ecb100000000000000000000000000000000000000000000000000011ac078002c01340325db7449f6fbdb410400000000000000000000000000000000000000ef503f81ae3fe501e3c75402000000000000000000000000000000000000000039818200ccf3fcecbf8cc71a901b55bd00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000c526000002250000";
        let encoded = hex_to_ba(encoded);
        let mut cursor = Cursor::new(encoded);
        let commitment = Commitment::read(&mut cursor).unwrap();

        assert_eq!(
            Commitment {
                stake: Some(Stake {
                    amount: BigUint::from_str("24041954535417099").unwrap(),
                    rewardable_amount: BigUint::from_str("24041954535417099").unwrap(),
                    created: 8_552_761,
                    cooldown_period: 28_800,
                    cooldown_started: None,
                    accrued_reward: BigUint::from_str("13878114491385").unwrap(),
                    accrued_slash: BigUint::from_str("0").unwrap(),
                    allow_auto_compound: true,
                    paid: BigUint::from_str("1306729452396152").unwrap(),
                    applied_slash: BigUint::from_str("0").unwrap(),
                }),
                commission: BigRational::new(BigInt::from(5), BigInt::from(100)),
                delegations_total_amount: BigUint::from_str("59252062669957319").unwrap(),
                delegations_total_rewardable_amount: BigUint::from_str("58524983623786623").unwrap(),
                weights: WeightsMemoryBuffer {
                    past: Some(WeightsMemoryBufferItem {
                        timestamp: 9_917,
                        weights: CommitmentWeights {
                            self_reward_weight: BigUint::from_str("22523954535417099").unwrap(),
                            self_slash_weight: BigUint::from_str("22523954535417099").unwrap(),
                            delegations_reward_weight: BigUint::from_str("49637271195029767").unwrap(),
                            delegations_slash_weight: BigUint::from_str("50047673331561616").unwrap(),
                        }
                    }),
                    current: WeightsMemoryBufferItem {
                        timestamp: 9_922,
                        weights: CommitmentWeights {
                            self_reward_weight: BigUint::from_str("24041954535417099").unwrap(),
                            self_slash_weight: BigUint::from_str("24041954535417099").unwrap(),
                            delegations_reward_weight: BigUint::from_str("49670386762010005").unwrap(),
                            delegations_slash_weight: BigUint::from_str("50080788898541854").unwrap(),
                        }
                    },
                },
                pool_rewards: PoolRewardsMemoryBuffer {
                    past: Some(PoolRewardsMemoryBufferItem {
                        timestamp: 7_913_498,
                        pool_reward: PoolReward {
                            reward_per_weight: BigUint::from_str("337295120307119722458126745900").unwrap(),
                            slash_per_weight: BigUint::from_str("721463726967900162020561135").unwrap(),
                        }
                    }),
                    current: PoolRewardsMemoryBufferItem {
                        timestamp: 8_552_761,
                        pool_reward: PoolReward {
                            reward_per_weight: BigUint::from_str("58595555713057311284724036556").unwrap(),
                            slash_per_weight: BigUint::from_str("0").unwrap(),
                        }
                    },
                },
                last_scoring_epoch: 9_925,
                last_slashing_epoch: 9_474,
            },
            commitment,
        );
    }
}
