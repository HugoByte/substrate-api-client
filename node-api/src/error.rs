// This file was taken from subxt (Parity Technologies (UK))
// https://github.com/paritytech/subxt/
// And was adapted by Supercomputing Systems AG and Integritee AG.
//
// Copyright 2019-2022 Parity Technologies (UK) Ltd, Supercomputing Systems AG and Integritee AG.
// This file is licensed as Apache-2.0
// see LICENSE for license details.

//! The errors use in the node-api crate.
//!
//! This file is mostly subxt.

use crate::{
	alloc::{
		borrow::Cow,
		format,
		string::{String, ToString},
		vec::Vec,
	},
	metadata::Metadata,
};
use codec::Decode;
use core::fmt::Debug;
use derive_more::From;
use log::*;
use scale_info::TypeDef;

// Re-expose the errors we use from other crates here:
pub use crate::{
	decoder::{DecodeError, EncodeError},
	metadata::{InvalidMetadataError, MetadataError},
};
pub use sp_core::crypto::SecretStringError;
pub use sp_runtime::transaction_validity::TransactionValidityError;

/// The underlying error enum, generic over the type held by the `Runtime`
/// variant. Prefer to use the [`Error<E>`] and [`Error`] aliases over
/// using this type directly.
#[derive(Debug, From)]
pub enum Error {
	/// Codec error.
	Codec(codec::Error),
	/// Serde serialization error
	Serialization(serde_json::error::Error),
	/// Secret string error.
	SecretString(SecretStringError),
	/// Extrinsic validity error
	Invalid(TransactionValidityError),
	/// Invalid metadata error
	InvalidMetadata(InvalidMetadataError),
	/// Invalid metadata error
	Metadata(MetadataError),
	/// Runtime error.
	Runtime(DispatchError),
	/// Error decoding to a [`crate::dynamic::Value`].
	DecodeValue(DecodeError),
	/// Error encoding from a [`crate::dynamic::Value`].
	EncodeValue(EncodeError<()>),
	/// Transaction progress error.
	Transaction(TransactionError),
	/// Block related error.
	Block(BlockError),
	/// An error encoding a storage address.
	StorageAddress(StorageAddressError),
	/// Other error.
	Other(String),
}

impl From<&str> for Error {
	fn from(error: &str) -> Self {
		Error::Other(error.into())
	}
}
/// This is our attempt to decode a runtime DispatchError. We either
/// successfully decode it into a [`ModuleError`], or we fail and keep
/// hold of the bytes, which we can attempt to decode if we have an
/// appropriate static type to hand.
#[derive(Debug)]
pub enum DispatchError {
	/// An error was emitted from a specific pallet/module.
	Module(ModuleError),
	/// Some other error was emitted.
	Other(Vec<u8>),
}

impl RuntimeError {
    /// Converts a `DispatchError` into a subxt error.
    pub fn from_dispatch(metadata: &Metadata, error: DispatchError) -> Result<Self, Error> {
        match error {
            DispatchError::Module(ModuleError {
                index,
                error,
                message: _,
                pallet,
                description,
                error_data,
            }) => {
                let error = metadata.error(index, error[0])?;
                Ok(Self::Module(PalletError {
                    pallet: error.pallet().to_string(),
                    error: error.error().to_string(),
                    description: error.description().to_vec(),
                }))
            }
            DispatchError::BadOrigin => Ok(Self::BadOrigin),
            DispatchError::CannotLookup => Ok(Self::CannotLookup),
            DispatchError::ConsumerRemaining => Ok(Self::ConsumerRemaining),
            DispatchError::TooManyConsumers => Ok(Self::TooManyConsumers),
            DispatchError::NoProviders => Ok(Self::NoProviders),
            DispatchError::Arithmetic(_math_error) => Ok(Self::Other("math_error".into())),
            DispatchError::Token(_token_error) => Ok(Self::Other("token error".into())),
            DispatchError::Transactional(_transactional_error) => {
                Ok(Self::Other("transactional error".into()))
            }
            DispatchError::Other(msg) => Ok(Self::Other(msg.to_string())),
            _ => todo!(),
        }
    }
}

/// Block error
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BlockError {
	/// The block
	BlockHashNotFound(String),
}

impl BlockError {
	/// Produce an error that a block with the given hash cannot be found.
	pub fn block_hash_not_found(hash: impl AsRef<[u8]>) -> BlockError {
		let hash = format!("0x{}", hex::encode(hash));
		BlockError::BlockHashNotFound(hash)
	}
}

/// Transaction error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransactionError {
	/// The finality subscription expired (after ~512 blocks we give up if the
	/// block hasn't yet been finalized).
	FinalitySubscriptionTimeout,
	/// The block hash that the transaction was added to could not be found.
	/// This is probably because the block was retracted before being finalized.
	BlockHashNotFound,
}

/// Details about a module error that has occurred.
#[derive(Clone, Debug)]
pub struct ModuleError {
	/// The name of the pallet that the error came from.
	pub pallet: String,
	/// The name of the error.
	pub error: String,
	/// A description of the error.
	pub description: Vec<String>,
	/// A byte representation of the error.
	pub error_data: ModuleErrorData,
}

/// The error details about a module error that has occurred.
///
/// **Note**: Structure used to obtain the underlying bytes of a ModuleError.
#[derive(Clone, Debug)]
pub struct ModuleErrorData {
	/// Index of the pallet that the error came from.
	pub pallet_index: u8,
	/// Raw error bytes.
	pub error: [u8; 4],
}

impl ModuleErrorData {
	/// Obtain the error index from the underlying byte data.
	pub fn error_index(&self) -> u8 {
		// Error index is utilized as the first byte from the error array.
		self.error[0]
	}
}

/// Something went wrong trying to encode a storage address.
#[derive(Clone, Debug)]
pub enum StorageAddressError {
	/// Storage map type must be a composite type.
	MapTypeMustBeTuple,
	/// Storage lookup does not have the expected number of keys.
	WrongNumberOfKeys {
		/// The actual number of keys needed, based on the metadata.
		actual: usize,
		/// The number of keys provided in the storage address.
		expected: usize,
	},
	/// Storage lookup requires a type that wasn't found in the metadata.
	TypeNotFound(u32),
	/// This storage entry in the metadata does not have the correct number of hashers to fields.
	WrongNumberOfHashers {
		/// The number of hashers in the metadata for this storage entry.
		hashers: usize,
		/// The number of fields in the metadata for this storage entry.
		fields: usize,
	},
}
