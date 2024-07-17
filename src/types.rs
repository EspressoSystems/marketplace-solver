use espresso_types::{FeeAmount, NamespaceId, SeqTypes};
use hotshot_types::traits::node_implementation::NodeType;

pub struct RollupRegistration {
    pub namespace_id: NamespaceId,
    // Denominated in Wei
    pub reserve_price: FeeAmount,
    // whether this registration is active in the marketplace
    pub active: bool,
    // a list of keys authorized to update the registration information
    pub signature_keys: Vec<<SeqTypes as NodeType>::SignatureKey>,
    // Optional field for human readable information
    pub text: String,
    // signature over the above data (must be from a key in the 'signature_keys` list)
    pub signature: <SeqTypes as NodeType>::SignatureKey,
}

pub struct RollupUpdate {
    pub namespace_id: Option<NamespaceId>,
    // Denominated in Wei
    pub reserve_price: Option<FeeAmount>,
    // whether this registration is active in the marketplace
    pub active: Option<bool>,
    // a list of keys authorized to update the registration information
    pub signature_keys: Vec<<SeqTypes as NodeType>::SignatureKey>,
    // Optional field for human readable information
    pub text: Option<String>,
    // signature over the above data (must be from a key in the 'signature_keys` list)
    pub signature: <SeqTypes as NodeType>::SignatureKey,
}
