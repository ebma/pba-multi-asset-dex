#![cfg(test)]

use crate as pallet_unique_items;
use crate as pallet_nft;
use frame_support::{
	parameter_types,
	traits::{ConstU16, ConstU32, ConstU64, Everything, GenesisBuild},
	PalletId,
};
use frame_system as system;
use orml_traits::parameter_type_with_key;
use primitives::{CurrencyId, TokenSymbol};
pub use primitives::{CurrencyId::Token, TokenSymbol::*, UnsignedInner};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, ConstU128, ConvertInto, IdentityLookup, Zero},
	BoundedVec, BuildStorage,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Tokens: orml_tokens::{Pallet, Call, Storage, Config<T>, Event<T>},

		Nfts: pallet_nft::{Pallet, Call, Storage, Config<T>, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}

parameter_types! {
	// One can owned at most 9,999 UniqueItems
	pub const MaxUniqueItemsOwned: u32 = 100;
	pub const StringLimit: u32 = 255;
}

pub type AssetId = CurrencyId;
pub type Balance = u128;
pub type ItemId = [u8; 16];

impl pallet_unique_items::Config for Test {
	type Event = Event;
	type Balance = Balance;
	type AssetId = AssetId;
	type ItemId = ItemId;
	type Assets = Tokens;
	type StringLimit = StringLimit;
	type MaxUniqueItemsOwned = MaxUniqueItemsOwned;
}

parameter_types! {
	pub const MaxLocks: u32 = 50;
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_a: AssetId| -> Balance {
		Zero::zero()
	};
}

impl orml_tokens::Config for Test {
	type Event = Event;
	type Balance = Balance;
	type Amount = primitives::Amount;
	type CurrencyId = AssetId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = ();
	type OnNewTokenAccount = ();
	type OnKilledTokenAccount = ();
	type MaxLocks = MaxLocks;
	type MaxReserves = ConstU32<0>;
	type ReserveIdentifier = [u8; 8];
	type DustRemovalWhitelist = Everything;
}

pub const ASSET_1: AssetId = CurrencyId::Token(TokenSymbol::Short([0; 4]));
pub const ASSET_2: AssetId = CurrencyId::Token(TokenSymbol::Short([1; 4]));

pub(crate) fn new_test_ext(users: Vec<(u64, [u8; 16], Vec<u8>)>) -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	GenesisConfig {
		tokens: TokensConfig {
			balances: users
				.iter()
				.flat_map(|(user, _, _)| {
					vec![
						(*user, CurrencyId::Native, 1_000_000),
						(*user, ASSET_1, 1_000_000),
						(*user, ASSET_2, 1_000_000),
					]
				})
				.collect(),
		},
		nfts: NftsConfig {
			unique_items: users
				.iter()
				.map(|(user, unique_item, data)| {
					(
						*user,
						*unique_item,
						BoundedVec::<u8, StringLimit>::truncate_from(data.clone()),
					)
				})
				.collect(),
		},
		..Default::default()
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
