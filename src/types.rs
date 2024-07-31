use committable::{Commitment, Committable};
use espresso_types::{FeeAmount, NamespaceId, SeqTypes};
use hotshot::types::SignatureKey;
use hotshot_types::traits::node_implementation::NodeType;
use serde::{Deserialize, Serialize};
use tide_disco::Url;

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct RollupRegistration {
    pub body: RollupRegistrationBody,
    // signature over the above data (must be from a key in the 'signature_keys` list)
    pub signature:
        <<SeqTypes as NodeType>::SignatureKey as SignatureKey>::PureAssembledSignatureType,
}

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct RollupRegistrationBody {
    pub namespace_id: NamespaceId,
    pub reserve_url: Url,
    // Denominated in Wei
    pub reserve_price: FeeAmount,
    // whether this registration is active in the marketplace
    pub active: bool,
    // a list of keys authorized to update the registration information
    pub signature_keys: Vec<<SeqTypes as NodeType>::SignatureKey>,
    // The signature key used to sign this registration body
    pub signature_key: <SeqTypes as NodeType>::SignatureKey,
    // Optional field for human readable information
    pub text: String,
}

impl Committable for RollupRegistrationBody {
    fn tag() -> String {
        "ROLLUP_REG".to_string()
    }

    fn commit(&self) -> Commitment<Self> {
        // todo (ab): expose to_fixed_bytes() method of fee amount
        let mut bytes = [0u8; 32];
        self.reserve_price.0.to_little_endian(&mut bytes);

        let active: [u8; 1] = if self.active { [1] } else { [0] };

        let mut comm = committable::RawCommitmentBuilder::new(&Self::tag())
            .u64_field("namespace_id", u64::from(self.namespace_id))
            .var_size_field("reserve_url", self.reserve_url.as_str().as_ref())
            .fixed_size_field("reserve_price", &bytes)
            .fixed_size_field("active", &active)
            .constant_str("signature_keys");

        for key in self.signature_keys.iter() {
            comm = comm.var_size_bytes(&key.to_bytes());
        }

        comm = comm
            .var_size_field("signature_key", &self.signature_key.to_bytes())
            .var_size_field("text", self.text.as_bytes());

        comm.finalize()
    }
}

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct RollupUpdate {
    pub body: RollupUpdatebody,
    // signature over the above data (must be from a key in the 'signature_keys` list)
    pub signature:
        <<SeqTypes as NodeType>::SignatureKey as SignatureKey>::PureAssembledSignatureType,
}

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct RollupUpdatebody {
    pub namespace_id: NamespaceId,
    // Denominated in Wei
    pub reserve_url: Option<Url>,
    pub reserve_price: Option<FeeAmount>,
    // whether this registration is active in the marketplace
    pub active: Option<bool>,
    // a list of keys authorized to update the registration information
    pub signature_keys: Option<Vec<<SeqTypes as NodeType>::SignatureKey>>,
    // The signature key used to sign this update body
    pub signature_key: <SeqTypes as NodeType>::SignatureKey,
    // Optional field for human readable information
    pub text: Option<String>,
}

impl Committable for RollupUpdatebody {
    fn tag() -> String {
        "ROLLUP_UPDATE".to_string()
    }

    fn commit(&self) -> Commitment<Self> {
        // todo (ab): expose to_fixed_bytes() method of fee amount

        let mut comm = committable::RawCommitmentBuilder::new(&Self::tag())
            .u64_field("namespace_id", u64::from(self.namespace_id));

        if let Some(reserve_url) = &self.reserve_url {
            comm = comm.var_size_field("reserve_url", reserve_url.as_str().as_ref())
        }

        if let Some(rp) = self.reserve_price {
            let mut bytes = [0u8; 32];
            rp.0.to_little_endian(&mut bytes);

            comm = comm.fixed_size_field("reserve_price", &bytes)
        };

        if let Some(active) = self.active {
            let active: [u8; 1] = if active { [1] } else { [0] };

            comm = comm.fixed_size_field("active", &active);
        }

        if let Some(keys) = &self.signature_keys {
            comm = comm.constant_str("signature_keys");
            for key in keys.iter() {
                comm = comm.var_size_bytes(&key.to_bytes());
            }
        }

        comm = comm.var_size_field("signature_key", &self.signature_key.to_bytes());

        if let Some(text) = &self.text {
            comm = comm.var_size_field("text", text.as_bytes());
        }

        comm.finalize()
    }
}
