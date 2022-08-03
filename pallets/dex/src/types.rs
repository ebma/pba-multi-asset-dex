


use crate::{
	traits::{Pool, PoolCreationParams},
	Config,
};

pub(crate) type BalanceOf<T> = <T as Config>::Balance;
pub(crate) type AssetIdOf<T> = <T as Config>::AssetId;
pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub(crate) type PoolOf<T> = Pool<AccountIdOf<T>, AssetIdOf<T>>;
pub(crate) type PoolCreationParamsOf<T> = PoolCreationParams<AccountIdOf<T>, AssetIdOf<T>>;
pub(crate) type PoolIdOf<T> = <T as Config>::PoolId;
