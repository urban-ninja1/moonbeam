// Copyright 2019-2022 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

//! A minimal precompile runtime including the pallet-randomness pallet
use super::*;
use codec::{Decode, Encode, MaxEncodedLen};
use pallet_evm::{
	IdentityAddressMapping, EnsureAddressNever, EnsureAddressRoot, Precompile, PrecompileSet,
};
use pallet_randomness::{Config, VrfInput};
use frame_support::{
	construct_runtime, parameter_types,
	traits::{Everything, GenesisBuild},
	weights::Weight,
};
use nimbus_primitives::NimbusId;
use serde::{Deserialize, Serialize};
use session_keys_primitives::VrfId;
use sp_consensus_babe::Slot;
use sp_core::{H160, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};
use sp_std::convert::{TryFrom, TryInto};
use precompile_utils::precompile_set::*;

pub type AccountId = H160;
pub type Balance = u128;
pub type BlockNumber = u64;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		AuthorMapping: pallet_author_mapping::{Pallet, Call, Storage, Config<T>, Event<T>},
		Evm: pallet_evm::{Pallet, Call, Storage, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		Randomness: pallet_randomness::{Pallet, Call, Storage, Event<T>, Inherent},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
	pub const SS58Prefix: u8 = 42;
}
impl frame_system::Config for Runtime {
	type BaseCallFilter = Everything;
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Call = Call;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type BlockWeights = ();
	type BlockLength = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 0;
}
impl pallet_balances::Config for Runtime {
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 4];
	type MaxLocks = ();
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

/// The randomness precompile is available at address one in the mock runtime.
pub fn precompile_address() -> H160 {
	H160::from_low_u64_be(1)
}

pub type TestPrecompiles<R> = PrecompileSetBuilder<
	R,
	(
		PrecompileAt<AddressU64<1>, RandomnessWrapper<R>, LimitRecursionTo<1>>,
		RevertPrecompile<AddressU64<2>>,
	),
>;

parameter_types! {
	pub PrecompilesValue: TestPrecompiles<Runtime> = TestPrecompiles::new();
}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = ();
	type GasWeightMapping = ();
	type CallOrigin = EnsureAddressRoot<Account>;
	type WithdrawOrigin = EnsureAddressNever<Account>;
	type AddressMapping = IdentityAddressMapping;
	type Currency = Balances;
	type Event = Event;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = TestPrecompiles<Runtime>;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = ();
	type OnChargeTransaction = ();
	type BlockGasLimit = ();
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type FindAuthor = ();
	type WeightInfo = ();
}

parameter_types! {
	pub const MinimumPeriod: u64 = 5;
}
impl pallet_timestamp::Config for Runtime {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const DepositAmount: Balance = 100;
}
impl pallet_author_mapping::Config for Runtime {
	type Event = Event;
	type DepositCurrency = Balances;
	type DepositAmount = DepositAmount;
	type Keys = VrfId;
	type WeightInfo = ();
}

pub struct BabeDataGetter;
impl pallet_randomness::traits::GetBabeData<BlockNumber, u64, Option<H256>> for BabeDataGetter {
	fn get_relay_epoch_index() -> u64 {
		1u64
	}
	fn get_epoch_randomness() -> Option<H256> {
		None
	}
}

pub struct VrfInputGetter;
impl pallet_randomness::traits::GetVrfInput<VrfInput<Slot, H256>> for VrfInputGetter {
	fn get_vrf_input() -> VrfInput<Slot, H256> {
		VrfInput::default()
	}
}

parameter_types! {
	pub const Deposit: u128 = 10;
	pub const ExpirationDelay: u32 = 5;
}
impl Config for Runtime {
	type Event = Event;
	type AddressMapping = IdentityAddressMapping;
	type Currency = Balances;
	type BabeDataGetter = BabeDataGetter;
	type VrfInputGetter = VrfInputGetter;
	type VrfKeyLookup = AuthorMapping;
	type Deposit = Deposit;
	type ExpirationDelay = ExpirationDelay;
	type WeightInfo = ();
}

pub(crate) fn events() -> Vec<pallet::Event<Runtime>> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| {
			if let Event::Randomness(inner) = e {
				Some(inner)
			} else {
				None
			}
		})
		.collect::<Vec<_>>()
}

/// Panics if an event is not found in the system log of events
#[macro_export]
macro_rules! assert_event_emitted {
	($event:expr) => {
		match &$event {
			e => {
				assert!(
					crate::mock::events().iter().find(|x| *x == e).is_some(),
					"Event {:?} was not found in events: \n {:?}",
					e,
					crate::mock::events()
				);
			}
		}
	};
}

/// Externality builder for pallet randomness mock runtime
pub(crate) struct ExtBuilder {
	/// Balance amounts per AccountId
	balances: Vec<(AccountId, Balance)>,
	/// AuthorId -> AccountId mappings
	mappings: Vec<(NimbusId, AccountId)>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder {
			balances: Vec::new(),
			mappings: Vec::new(),
		}
	}
}

impl ExtBuilder {
	#[allow(dead_code)]
	pub(crate) fn with_balances(mut self, balances: Vec<(Account, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	#[allow(dead_code)]
	pub(crate) fn with_mappings(mut self, mappings: Vec<(NimbusId, Account)>) -> Self {
		self.mappings = mappings;
		self
	}

	#[allow(dead_code)]
	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<Runtime> {
			balances: self.balances,
		}
		.assimilate_storage(&mut t)
		.expect("Pallet balances storage can be assimilated");

		pallet_author_mapping::GenesisConfig::<Runtime> {
			mappings: self.mappings,
		}
		.assimilate_storage(&mut t)
		.expect("Pallet author mapping's storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
