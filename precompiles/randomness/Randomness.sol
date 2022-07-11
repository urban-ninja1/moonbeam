// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

/// @dev Maximum number of random words being requested
uint32 constant MAX_RANDOM_WORDS = 100;
/// @dev Minimum number of blocks before a request can be fulfilled
uint32 constant MIN_DELAY_BLOCKS = 1;
/// @dev Maximum number of blocks before a request can be fulfilled
uint32 constant MAX_DELAY_BLOCKS = 10000;

interface Randomness {
    /// @notice The status of the request
    /// @param Pending The request cannot be fulfilled yet
    /// @param Ready The request is ready to be fulfilled
    /// @param Expired The request has expired
    enum RequestStatus {
        Pending,
        Ready,
        Expired
    }

    /// @notice The type of randomness source
    /// @param LocalVRF Randomness VRF using the parachain material as seed
    /// @param RelayBabeEpoch Randomness VRF using relay material from previous epoch
    enum RandomnessSource {
        LocalVRF,
        RelayBabeEpoch
    }

    /// @notice The request details
    /// @param id The id of the request
    /// @param refundAddress The address receiving the left-over fees after the fulfillment
    /// @param contractAddress The address of the contract being called back during fulfillment
    /// @param fee The amount to set aside to pay for the fulfillment
    /// @param gasLimit The gas limit to use for the fulfillment
    /// @param salt A string being mixed with the randomness seed to obtain different random words
    /// @param numWords The number of random words requested (from 1 to MAX_RANDOM_WORDS)
    /// @param randomnessSource The type of randomness source used to generate the random words
    /// @param fulfillmentBlock The parachain block number at which the request can be fulfilled (for LocalVRF only)
    /// @param fulfillmentEpochIndex The relay epoch index at which the request can be fulfilled (for RelayBabeEpoch)
    /// @param expirationBlock The parachain block number at which the request expires
    /// @param status The current status of the request
    struct Request {
        uint256 id;
        address refundAddress;
        address contractAddress;
        uint256 fee;
        uint256 gasLimit;
        bytes32 salt;
        uint32 numWords;
        RandomnessSource randomnessSource;
        uint32 fulfillmentBlock;
        uint32 fulfillmentEpochIndex;
        uint32 expirationBlock;
        RequestStatus status;
    }

    /// Return the current relay epoch index
    /// @dev An epoch represents real time and not a block number
    /// @dev Currently, it cannot be longer than:
    /// @dev  - Kusama/Westend/Rococo: 600 relay blocks (1 hour)
    /// @dev  - Polkadot: 2400 relay blocks (4 hours)
    /// Selector: 81797566
    function relayEpochIndex() external view returns (uint64);

    /// @param requestId The id of the request to check
    /// @return request The request
    /// Selector: c58343ef
    function getRequest(uint256 requestId)
        external
        view
        returns (Request memory request);

    /// @notice Request random words generated from the parachain VRF
    /// @dev This is using pseudo-random VRF executed by the collator at the fulfillment
    /// @dev Warning:
    /// @dev The collator in charge of producing the block at fulfillment can decide to skip
    /// @dev producing the block in order to have a different random word generated by the next
    /// @dev collator, at the cost of a block reward. It is therefore economically viable to use
    /// @dev this randomness source only if the financial reward at stake is lower than the block
    /// @dev reward.
    /// @dev In order to reduce the risk of a collator being able to predict the random words
    /// @dev when the request is performed, it is possible to increase the delay to multiple blocks
    /// @dev The higher the delay is, the less likely the collator will be able to know which
    /// @dev collator will be in charge of fulfilling the request.
    /// @dev Fulfillment is manual and can be executed by anyone (for free) after the given delay
    /// @param refundAddress The address receiving the left-over fees after the fulfillment
    /// @param fee The amount to set aside to pay for the fulfillment
    /// @param gasLimit The gas limit to use for the fulfillment
    /// @param salt A string being mixed with the randomness seed to obtain different random words
    /// @param numWords The number of random words requested (from 1 to MAX_RANDOM_WORDS)
    /// @param delay The number of blocks until the request can be fulfilled (between MIN_DELAY_BLOCKS and MAX_DELAY_BLOCKS)
    /// @return requestId The id of the request requestLocalVRFRandomWords
    /// Selector: 4c9c81a8
    function requestLocalVRFRandomWords(
        address refundAddress,
        uint256 fee,
        uint64 gasLimit,
        bytes32 salt,
        uint64 numWords,
        uint64 delay
    ) external returns (uint256);

    /// @notice Request random words generated from the relaychain Babe consensus
    /// @dev The random words are generated from the hash of the all the VRF provided by the
    /// @dev relaychain validator during 1 epoch.
    /// @dev It requires a delay of at least 1 epoch after the current epoch to be unpredictable
    /// @dev at the time the request is performed.
    /// @dev Warning:
    /// @dev The validator (on the relaychain) of the last block of an epoch can decide to skip
    /// @dev producing the block in order to choose the previous generated epoch random number
    /// @dev at the cost of a relaychain block rewards. It is therefore economically viable to use
    /// @dev this randomness source only if the financial reward at stake is lower than the relaychain
    /// @dev block reward.
    /// @dev (see https://crates.parity.io/pallet_babe/struct.RandomnessFromOneEpochAgo.html)
    /// @dev Fulfillment is manual and can be executed by anyone (for free) at
    /// @dev the beginning of the 2nd relay epoch following the current one
    /// @param refundAddress The address receiving the left-over fees after the fulfillment
    /// @param fee Amount to set aside to pay for the fulfillment. Those fees are taken from the contract
    /// @param gasLimit Gas limit for the fulfillment
    /// @param salt Salt to be mixed with raw randomness to get output
    /// @param numWords Number of random words to be returned (limited to MAX_RANDOM_WORDS)
    /// @return requestId The id of the request
    /// Selector: bd063318
    function requestRelayBabeEpochRandomWords(
        address refundAddress,
        uint256 fee,
        uint64 gasLimit,
        bytes32 salt,
        uint64 numWords
    ) external returns (uint256);

    /// @dev fulFill the request which will call the contract method "fulfillRandomWords"
    /// @dev Fees of the caller are refunded if the request is fulfillable
    /// @param requestId Request to be fulfilled
    /// Selector: 9a91eb0d
    function fulfillRequest(uint256 requestId) external;

    /// @param requestId Request receiving the additional fees
    /// @param feeIncrease Amount to increase
    /// Selector: d0408a7f
    function increaseRequestFee(uint256 requestId, uint256 feeIncrease)
        external;

    /// @param requestId Request to be purged
    /// Selector: 1d26cbab
    function purgeExpiredRequest(uint256 requestId) external;
}
