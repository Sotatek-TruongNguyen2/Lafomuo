const ONCHAINID = require("@onchain-id/solidity");
const { Identity, IdentitySDK } = require("@onchain-id/identity-sdk");
const { ethers } = require('ethers');

(async () => {
    const provider = ethers.getDefaultProvider('goerli');
    const signer = new ethers.Wallet('d970542f41d49c7cdcfbfddedd22a378094c47258ef1dbaaa4ee0103dcc0be11', provider);
    
    const identity = await IdentitySDK.Identity.at('0xe1c0cd22d92b6b445e50750b0f6470b67306e8ec', { provider });
    console.log(identity);

    const addKeyTransaction = await identity.addKey(IdentitySDK.utils.encodeAndHash(['address'], ["0xe1c0cd22d92b6b445e50750b0f6470b67306e8ec"]), IdentitySDK.utils.enums.KeyPurpose.MANAGEMENT, IdentitySDK.utils.enums.KeyType.ECDSA, { signer });
    console.log(addKeyTransaction);
})()