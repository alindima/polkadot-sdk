// Copyright (C) Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

//! Requests and responses as sent over the wire for the individual protocols.

use parity_scale_codec::{Decode, Encode};

use polkadot_node_primitives::{ErasureChunk, Proof};
use polkadot_primitives::{
	CandidateHash, ChunkIndex, CommittedCandidateReceipt, Hash, Id as ParaId,
	PersistedValidationData, UncheckedSignedStatement,
};

use super::{v1, IsRequest, Protocol};
use crate::v2::StatementFilter;

/// Request a candidate with statements.
#[derive(Debug, Clone, Encode, Decode)]
pub struct AttestedCandidateRequest {
	/// Hash of the candidate we want to request.
	pub candidate_hash: CandidateHash,
	/// Statement filter with 'OR' semantics, indicating which validators
	/// not to send statements for.
	///
	/// The filter must have exactly the minimum size required to
	/// fit all validators from the backing group.
	///
	/// The response may not contain any statements masked out by this mask.
	pub mask: StatementFilter,
}

/// Response to an `AttestedCandidateRequest`.
#[derive(Debug, Clone, Encode, Decode)]
pub struct AttestedCandidateResponse {
	/// The candidate receipt, with commitments.
	pub candidate_receipt: CommittedCandidateReceipt,
	/// The [`PersistedValidationData`] corresponding to the candidate.
	pub persisted_validation_data: PersistedValidationData,
	/// All known statements about the candidate, in compact form,
	/// omitting `Seconded` statements which were intended to be masked
	/// out.
	pub statements: Vec<UncheckedSignedStatement>,
}

impl IsRequest for AttestedCandidateRequest {
	type Response = AttestedCandidateResponse;
	const PROTOCOL: Protocol = Protocol::AttestedCandidateV2;
}

/// Responses as sent by collators.
pub type CollationFetchingResponse = super::v1::CollationFetchingResponse;

/// Request the advertised collation at that relay-parent.
#[derive(Debug, Clone, Encode, Decode)]
pub struct CollationFetchingRequest {
	/// Relay parent collation is built on top of.
	pub relay_parent: Hash,
	/// The `ParaId` of the collation.
	pub para_id: ParaId,
	/// Candidate hash.
	pub candidate_hash: CandidateHash,
}

impl IsRequest for CollationFetchingRequest {
	// The response is the same as for V1.
	type Response = CollationFetchingResponse;
	const PROTOCOL: Protocol = Protocol::CollationFetchingV2;
}

/// Request an availability chunk.
#[derive(Debug, Copy, Clone, Encode, Decode)]
pub struct ChunkFetchingRequest {
	/// Hash of candidate we want a chunk for.
	pub candidate_hash: CandidateHash,
	/// The index of the chunk to fetch.
	pub index: ChunkIndex,
}

/// Receive a requested erasure chunk.
#[derive(Debug, Clone, Encode, Decode)]
pub enum ChunkFetchingResponse {
	/// The requested chunk data.
	#[codec(index = 0)]
	Chunk(ChunkResponse),
	/// Node was not in possession of the requested chunk.
	#[codec(index = 1)]
	NoSuchChunk,
}

impl From<Option<ChunkResponse>> for ChunkFetchingResponse {
	fn from(x: Option<ChunkResponse>) -> Self {
		match x {
			Some(c) => ChunkFetchingResponse::Chunk(c),
			None => ChunkFetchingResponse::NoSuchChunk,
		}
	}
}

impl From<v1::ChunkFetchingRequest> for ChunkFetchingRequest {
	fn from(v1::ChunkFetchingRequest { candidate_hash, index }: v1::ChunkFetchingRequest) -> Self {
		Self { candidate_hash, index }
	}
}

impl From<ChunkFetchingRequest> for v1::ChunkFetchingRequest {
	fn from(ChunkFetchingRequest { candidate_hash, index }: ChunkFetchingRequest) -> Self {
		Self { candidate_hash, index }
	}
}

/// Skimmed down variant of `ErasureChunk`.
///
/// Instead of transmitting a full `ErasureChunk` we transmit `ChunkResponse` in
/// `ChunkFetchingResponse`, which omits the chunk's index. The index is already known by
/// the requester and by not transmitting it, we ensure the requester is going to use his index
/// value for validating the response, thus making sure he got what he requested.
#[derive(Debug, Clone, Encode, Decode)]
pub struct ChunkResponse {
	/// The erasure-encoded chunk of data belonging to the candidate block.
	pub chunk: Vec<u8>,
	/// Proof for this chunk's branch in the Merkle tree.
	pub proof: Proof,
	/// The index of the fetched chunk.
	pub index: ChunkIndex,
}

impl From<ErasureChunk> for ChunkResponse {
	fn from(ErasureChunk { chunk, index, proof }: ErasureChunk) -> Self {
		ChunkResponse { chunk, proof, index }
	}
}

impl From<ChunkResponse> for ErasureChunk {
	fn from(value: ChunkResponse) -> Self {
		Self { chunk: value.chunk, proof: value.proof, index: value.index }
	}
}

impl IsRequest for ChunkFetchingRequest {
	type Response = ChunkFetchingResponse;
	const PROTOCOL: Protocol = Protocol::ChunkFetchingV2;
}
