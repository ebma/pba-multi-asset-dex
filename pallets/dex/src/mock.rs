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
	traits::{BlakeTwo256, ConvertInto, IdentityLookup, Zero},
};

use crate as pallet_dex;

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
		Tokens: orml_tokens::{Pallet, Storage, Config<T>, Event<T>},

		Dex: pallet_dex::{Pallet, Call, Storage, Event<T>},
	}
);

impl system::Config for Test {
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
	type BlockHashCount = ConstU64<250>;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

pub type AccountId = u64;
pub type AssetId = CurrencyId;
pub type Balance = u128;
pub type BlockNumber = u64;
pub type PoolId = u128;
pub type Moment = u64;
pub type Index = u64;

parameter_types! {
	pub const GetNativeCurrencyId: AssetId = CurrencyId::Native;
	pub const MaxLocks: u32 = 50;
	pub const DexPalletId: PalletId = PalletId(*b"dex_pall");
}

impl pallet_dex::Config for Test {
	type Event = Event;
	type Balance = Balance;
	type AssetId = AssetId;
	type PoolId = PoolId;
	type PalletId = DexPalletId;
	type Assets = Tokens;
	type Convert = ConvertInto;
	type LiquidityTokenConversion = primitives::token_conversion::CurrencyConversion;
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

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARLIE: AccountId = 3;
pub const DARWIN: AccountId = 4;

pub const ASSET_1: AssetId = CurrencyId::Token(TokenSymbol::Short([0; 4]));
pub const ASSET_2: AssetId = CurrencyId::Token(TokenSymbol::Short([1; 4]));
// pub const ASSET_1: AssetId = CurrencyId::Token(TokenSymbol::Short([0x00, 0x01, 0x02, 0x03]));


pub fn new_test_ext_multi_currency() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into();

	let base_balance = 1_000_000;

	let balances: Vec<(AccountId, AssetId, Balance)> = vec![
		(ALICE, ASSET_1, base_balance),
		(ALICE, ASSET_2, base_balance),
		(BOB, ASSET_1, base_balance),
	];

	orml_tokens::GenesisConfig::<Test> { balances }
		.assimilate_storage(&mut t)
		.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	// set block number to 1 to make sure that events are populated
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub fn run_test<T>(test: T)
where
	T: FnOnce(),
{
	new_test_ext_multi_currency().execute_with(|| {
		test();
	});
}
