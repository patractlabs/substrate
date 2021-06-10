/// A wrapped Cid for source Cid struct, to implement a valid encode/decode for Cid.
///
/// This file will be used until this pr is merged:
/// https://github.com/multiformats/rust-multihash/pull/116
/// For now, the source Cid use a static length `U64(H256)` for all Cid type, if we use the source
/// Cid encode/decode, it will add lots of empty zero in the result.
///
/// e.g.
/// In the most widely used Cid type in IPFS, like:
/// QmRZdc3mAMXpv6Akz9Ekp1y4vDSjazTx2dCQRkxVy1yUj6
/// If we use the source Cid to do encode, it will be:
/// 00-7000000000000000-1200000000000000-20-2fe65ccc17fe180c3bf4e9b8490fcc6dc74c30bf6595795dcd1136d8d9cb3f950000000000000000000000000000000000000000000000000000000000000000
/// |Version|-|codec:u64|-|MultiHash|
///                       |codec:u64| - |size:u8| - |digest|
/// We can see that ths digest part contains lots of zero for current digest in MultiHash is `[u8; 64]`,
/// However the generic hash length is 32. So the default encode/decode method wastes a lot of space.
///
/// And in pr#116 which is list above, the static length will be changed for multihash. Thus the
/// encode/decode method will be suitable for different Cid type.
/// So we consider this pr, decide to implement a encode/decode method for Cid which will be
/// **compatible** with the modification in pr#116.
///
/// In our encode/decode, for the last part `digest`, we write the **raw value** to buffer and
/// read it from buffer **directly**, and do not add other byte like hint size or else.
/// The `code` and `size` is encoded/decoded in normal way.
///
use codec::{Decode, Encode, EncodeLike, Error, Input, Output};
use rust_cid::Cid as SourceCid;

use sp_runtime::RuntimeDebug;
use sp_std::{ops::Deref, vec};

#[derive(PartialEq, Eq, Clone, PartialOrd, Ord, Hash, Copy)] // SourceCid has implemeted the Copy for static length.
#[derive(RuntimeDebug)]
pub struct Cid(SourceCid);

impl Cid {
	pub fn new(cid: SourceCid) -> Self {
		Self(cid)
	}
}

impl Encode for Cid {
	fn encode_to<EncOut: Output + ?Sized>(&self, dest: &mut EncOut) {
		// for cid
		self.version().encode_to(dest);
		self.codec().encode_to(dest);
		// for multihash
		let hash = self.hash();
		let code = hash.code();
		let size = hash.size();
		let digest = hash.digest();

		code.encode_to(dest);
		size.encode_to(dest);
		// notice we write the digest directly to dest, for we have known the size.
		// **IMPORTANT**
		// we do not choose to encode &[u8] directly, for it will add compact length at start.
		//
		// in a valid cid, digest.len() must equal to `size`. Thus, in Decode,
		// we can just read a raw bytes which length is equal to `size`.
		dest.write(digest)
	}
}

impl EncodeLike for Cid {}

impl Decode for Cid {
	fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
		use rust_cid::{multihash, Version};
		type Multihash = multihash::MultihashGeneric<multihash::U64>;

		// for cid
		let version: Version = Decode::decode(input)?;
		let codec: u64 = Decode::decode(input)?;
		// for multihash
		let code: u64 = Decode::decode(input)?;
		let size: u8 = Decode::decode(input)?;
		let mut buf = vec![0; size as usize];
		// In a valid Cid, the size must equal to this raw buffer.
		input.read(&mut buf)?;
		let hash = Multihash::wrap(code, &buf).map_err(|_| "Multihash parse error")?;
		Ok(Cid::new(
			SourceCid::new(version, codec, hash).map_err(|_| "Cid parse error")?,
		))
	}
}

impl Into<SourceCid> for Cid {
	fn into(self) -> SourceCid {
		self.0
	}
}

impl From<SourceCid> for Cid {
	fn from(cid: SourceCid) -> Self {
		Cid::new(cid)
	}
}

impl AsRef<SourceCid> for Cid {
	fn as_ref(&self) -> &SourceCid {
		&self.0
	}
}

impl AsMut<SourceCid> for Cid {
	fn as_mut(&mut self) -> &mut SourceCid {
		&mut self.0
	}
}

impl Deref for Cid {
	type Target = SourceCid;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::str::FromStr;
	#[test]
	fn normal_test_for_example() {
		let s = "QmRZdc3mAMXpv6Akz9Ekp1y4vDSjazTx2dCQRkxVy1yUj6";
		let cid: Cid = SourceCid::from_str(s).expect("must be valid.").into();
		let bytes = cid.encode();
		let r = hex::encode(&bytes);
		let expect = "0070000000000000001200000000000000202fe65ccc17fe180c3bf4e9b8490fcc6dc74c30bf6595795dcd1136d8d9cb3f95";
		assert_eq!(r, expect);
		let new_cid: Cid = Decode::decode(&mut &bytes[..]).expect("must decode well");
		assert_eq!(new_cid, cid);
	}
	// those test case is from crate rust-cid
}
